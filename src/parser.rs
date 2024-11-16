use std::{iter::Peekable, str::Chars};

pub enum Value {
  SimpleString(String),
}

pub struct Parser;

impl Parser {
    pub fn parse(input: &str) -> Result<Value, String> {
        let mut chars = input.chars().peekable();
        match chars.next() {
            Some('+') => Ok(Self::parse_simple_string(&mut chars)?),
            // Some('-') => Ok(Self::parse_error(&mut chars)?),
            // Some(':') => Ok(Self::parse_integer(&mut chars)?),
            // Some('$') => Ok(Self::parse_bulk_string(&mut chars)?),
            // Some('*') => Ok(Self::parse_array(&mut chars)?),
            _ => Err("Invalid RESP data".to_string()),
        }
    }
    fn parse_simple_string(chars: &mut Peekable<Chars>) -> Result<Value, String> {
        let mut buf = String::new();
        while let Some(c) = chars.next() {
            if c == '\r' {
                chars.next(); // Consume '\n'
                return Ok(Value::SimpleString(buf));
            }
            buf.push(c);
        }
        Err("Incomplete RESP data".to_string())
    }
}
