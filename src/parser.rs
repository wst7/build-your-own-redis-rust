use std::fmt::{Display, Formatter, Result as FmtResult};
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::str;

/// 定义 RESP 类型
#[derive(Debug)]
pub enum RespType {
    SimpleString(String),
    Error(String),
    Integer(i64),
    BulkString(Option<String>),
    Array(Option<Vec<RespType>>),
}

// impl Display for RespType {
//   fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
//       match self {
//           RespType::SimpleString(s) => write!(f, "+{}\r\n", s),
//           RespType::Error(s) => write!(f, "-{}\r\n", s),
//           RespType::Integer(i) => write!(f, ":{}\r\n", i),
//           RespType::BulkString(Some(s)) => write!(f, "${}\r\n{}\r\n", s.len(), s),
//           RespType::BulkString(None) => write!(f, "$-1\r\n"),
//           RespType::Array(Some(elements)) => {
//               write!(f, "*{}\r\n", elements.len())?;
//               for element in elements {
//                   write!(f, "{}", element)?;
//               }
//               Ok(())
//           }
//           RespType::Array(None) => write!(f, "*-1\r\n"),
//       }
//   }
// }

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
            b'-' => self.parse_error(),
            b':' => self.parse_integer(),
            b'$' => self.parse_bulk_string(),
            b'*' => self.parse_array(),
            _ => Err("Invalid RESP type marker".to_string()),
        }
    }

    fn parse_simple_string(&mut self) -> Result<RespType, String> {
        let line = self.read_line()?;
        Ok(RespType::SimpleString(line))
    }

    fn parse_error(&mut self) -> Result<RespType, String> {
        let line = self.read_line()?;
        Ok(RespType::Error(line))
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
