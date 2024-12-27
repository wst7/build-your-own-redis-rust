use num_bigint::BigInt;
use std::fmt::{Display, Formatter, Result as FmtResult};
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::str;

/// 定义 RESP 类型
#[derive(Debug)]
pub enum RespType {
    SimpleString(String),
    SimpleError(String),
    Integer(i64),
    BulkString(Option<String>),
    BulkError(String),
    Array(Option<Vec<RespType>>),
    Null,
    Boolean(bool),
    Double(f64),
    BigNumber(BigInt),
}

impl RespType {
  pub fn serialize(&self) -> Vec<u8> {
    match self {
      RespType::SimpleString(s) => format!("+{}\r\n", s).into_bytes(),
      RespType::SimpleError(e) => format!("-{}\r\n", e).into_bytes(),
      RespType::Integer(i) => format!(":{}\r\n", i).into_bytes(),
      RespType::BulkString(Some(s)) => {
        format!("${}\r\n{}\r\n", s.len(), s).into_bytes()
      }
      RespType::BulkString(None) => "$-1\r\n".to_string().into_bytes(),
      RespType::BulkError(e) => format!("!{}\r\n", e).into_bytes(),
      RespType::Array(Some(elements)) => {
        let mut serialized = format!("*{}\r\n", elements.len()).into_bytes();
        
        for element in elements {
          serialized.extend(element.serialize());
        }
        serialized
      }
      RespType::Array(None) => "*-1\r\n".to_string().into_bytes(),
      RespType::Null => "_\r\n".to_string().into_bytes(),
      RespType::Boolean(b) => format!("#{}\r\n", if *b { "t" } else { "f" }).into_bytes(),
      RespType::Double(d) => format!(",{}\r\n", d).into_bytes(),
      RespType::BigNumber(n) => format!("({})\r\n", n).into_bytes(),

    }
  }
}


/// RESP 解析器
pub struct RespParser<'a> {
    input: &'a [u8],
    pos: usize,
}

impl<'a> RespParser<'a> {
    /// 创建解析器
    pub fn new(input: &'a [u8]) -> Self {
        Self { input, pos: 0 }
    }

    /// 解析一条 RESP 消息
    pub fn parse(&mut self) -> Result<RespType, String> {
        if self.pos >= self.input.len() {
            return Err("Input exhausted".to_string());
        }

        let prefix = self.input[self.pos];
        self.pos += 1;

        match prefix {
            b'+' => self.parse_simple_string(),
            b'-' => self.parse_simple_error(),
            b':' => self.parse_integer(),
            b'$' => self.parse_bulk_string(),
            b'!' => self.parse_bulk_error(),
            b'*' => self.parse_array(),
            b'_' => self.parse_null(),
            b'#' => self.parse_boolean(),
            b',' => self.parse_double(),
            b'(' => self.parse_big_number(),
            _ => Err("Invalid RESP type marker".to_string()),
        }
    }

    fn parse_simple_string(&mut self) -> Result<RespType, String> {
        let line = self.read_line()?;
        Ok(RespType::SimpleString(line))
    }

    fn parse_simple_error(&mut self) -> Result<RespType, String> {
        let line = self.read_line()?;
        Ok(RespType::SimpleError(line))
    }

    fn parse_integer(&mut self) -> Result<RespType, String> {
        let line = self.read_line()?;
        let number: i64 = line.parse().map_err(|_| "Invalid integer".to_string())?;
        Ok(RespType::Integer(number))
    }

    fn parse_bulk_string(&mut self) -> Result<RespType, String> {
        let length: isize = self
            .read_line()?
            .parse()
            .map_err(|_| "Invalid bulk string length".to_string())?;

        if length == -1 {
            return Ok(RespType::BulkString(None));
        }

        let start = self.pos;
        let end = self.pos + length as usize;

        if end + 2 > self.input.len() || self.input[end..end + 2] != *b"\r\n" {
            return Err("Invalid bulk string termination".to_string());
        }

        self.pos = end + 2;

        let string = str::from_utf8(&self.input[start..end])
            .map_err(|_| "Invalid UTF-8 in bulk string".to_string())?
            .to_string();

        Ok(RespType::BulkString(Some(string)))
    }

    fn parse_bulk_error(&mut self) -> Result<RespType, String> {
        let length: isize = self
            .read_line()?
            .parse()
            .map_err(|_| "Invalid bulk error length".to_string())?;
        let start = self.pos;
        let end = self.pos + length as usize;
        if end + 2 > self.input.len() || self.input[end..end + 2] != *b"\r\n" {
            return Err("Invalid bulk error termination".to_string());
        }
        let error = str::from_utf8(&self.input[start..end])
            .map_err(|_| "Invalid UTF-8 in bulk error")?
            .to_string();
        self.pos = end + 2;
        Ok(RespType::BulkError(error))
    }

    fn parse_array(&mut self) -> Result<RespType, String> {
        let length: isize = self
            .read_line()?
            .parse()
            .map_err(|_| "Invalid array length".to_string())?;

        if length == -1 {
            return Ok(RespType::Array(None));
        }

        let mut elements = Vec::new();
        for _ in 0..length {
            elements.push(self.parse()?);
        }

        Ok(RespType::Array(Some(elements)))
    }

    fn parse_null(&mut self) -> Result<RespType, String> {
        let line = self.read_line()?;
        if line != "" {
            return Err("Invalid null value".to_string());
        }
        Ok(RespType::Null)
    }

    fn parse_boolean(&mut self) -> Result<RespType, String> {
        let value = self.read_line()?;
        let bool = match value.as_str() {
            "t" => true,
            "f" => false,
            _ => return Err("Invalid boolean value".to_string()),
        };
        return Ok(RespType::Boolean(bool));
    }

    fn parse_double(&mut self) -> Result<RespType, String> {
        let value = self.read_line()?;
        let double: f64 = value
            .parse()
            .map_err(|_| "Invalid double value".to_string())?;
        Ok(RespType::Double(double))
    }

    fn parse_big_number(&mut self) -> Result<RespType, String> {
        let value = self.read_line()?;
        let big_num: BigInt = value
            .parse()
            .map_err(|_| "Invalid big number value".to_string())?;
        Ok(RespType::BigNumber(big_num))
    }

    fn read_line(&mut self) -> Result<String, String> {
        if let Some(pos) = self.input[self.pos..].iter().position(|&b| b == b'\n') {
            let line = &self.input[self.pos..self.pos + pos - 1]; // 去掉 \r
            self.pos += pos + 1; // 跳过 \r\n
            str::from_utf8(line)
                .map(|s| s.to_string())
                .map_err(|_| "Invalid UTF-8 in line".to_string())
        } else {
            Err("Line not terminated".to_string())
        }
    }
}
