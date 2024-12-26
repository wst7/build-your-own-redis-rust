use std::{
    collections::HashMap,
    fmt::{self, Display, Formatter},
    io::{Cursor, Read},
    path::PathBuf,
    str, vec,
};

use byteorder::{LittleEndian, ReadBytesExt};
// use tokio::io::AsyncReadExt;

enum OpCode {
    AUX = 0xFA,
    RESIZEDB = 0xFB,
    EXPIRETIMEMS = 0xFC,
    EXPIRETIME = 0xFD,
    SELECTDB = 0xFE,
    EOF = 0xFF,
    Unknown,
}
enum RdValueType {
    String = 0,
    List = 1,
    Set = 2,
    SortedSet = 3,
    Hash = 4,
    ZipMap = 9,
    ZipList = 10,
    IntSet = 11,
    SortedSetInZipList = 12,
    HashMapInZipList = 13,
    ZipInQuickList = 14,
    Unknown,
}
enum RdLength {
    Integer(u8),
    Len(u32),
    LZF,
}
#[derive(Debug)]
enum RdbString {
    String(Vec<u8>),
    Integer(Vec<u8>),
    LZF(Vec<u8>),
}
impl ToString for RdbString {
    fn to_string(&self) -> String {
        match self {
            RdbString::String(s) => String::from_utf8_lossy(s).to_string(),
            RdbString::Integer(i) => String::from_utf8_lossy(i).to_string(),
            RdbString::LZF(_) => "LZF".to_string(),
        }
    }
}

impl From<u8> for OpCode {
    fn from(v: u8) -> Self {
        match v {
            0xFA => OpCode::AUX,
            0xFB => OpCode::RESIZEDB,
            0xFC => OpCode::EXPIRETIMEMS,
            0xFD => OpCode::EXPIRETIME,
            0xFE => OpCode::SELECTDB,
            0xFF => OpCode::EOF,
            _ => OpCode::Unknown,
        }
    }
}
impl From<u8> for RdValueType {
    fn from(v: u8) -> Self {
        match v {
            0 => RdValueType::String,
            1 => RdValueType::List,
            2 => RdValueType::Set,
            3 => RdValueType::SortedSet,
            4 => RdValueType::Hash,
            9 => RdValueType::ZipMap,
            10 => RdValueType::ZipList,
            11 => RdValueType::IntSet,
            12 => RdValueType::SortedSetInZipList,
            13 => RdValueType::HashMapInZipList,
            14 => RdValueType::ZipInQuickList,
            _ => RdValueType::Unknown,
        }
    }
}
pub struct RdbParser {
    input: Vec<u8>,
    pos: usize,
    handler: fn(db: usize, key: String, value: String, expire: Option<u128>),
}
impl RdbParser {
    pub fn new(
        input: Vec<u8>,
        handler: fn(db: usize, key: String, value: String, expire: Option<u128>),
    ) -> Self {
        Self {
            input,
            pos: 0,
            handler,
        }
    }
    pub fn parse(&mut self) -> Result<Rdb, String> {
        let header = self.parse_header()?;
        let mut rdb = Rdb {
            header,
            metadata: RdbMetadata {
                info: HashMap::new(),
            },
            // databases: vec![],
            checksum: 0,
        };

        let mut expires_at = None;
        let mut db_index: usize = 0;

        loop {
            let next_op = self.read_byte()?;
            match OpCode::from(next_op) {
                OpCode::AUX => {
                    let key = self.read_string()?;
                    let value = self.read_string()?;
                    rdb.metadata.info.insert(key.to_string(), value.to_string());
                }
                OpCode::EOF => {
                    // TODO: checksum
                    let _ = self.read_bytes(8)?;
                    break;
                }
                OpCode::SELECTDB => {
                    let db_number = self.read_length()?;
                    match db_number {
                        RdLength::Len(num) => {
                            db_index = num as usize;
                        }
                        _ => return Err("Invalid db number".to_string()),
                    };
                }
                OpCode::RESIZEDB => {
                    let db_size = self.read_length()?;
                    let expires_size = self.read_length()?;
                }
                OpCode::EXPIRETIME => {
                    let timestamp = self.read_bytes(4)?;
                    let mut rdr = Cursor::new(timestamp);
                    expires_at = Some(
                        (rdr.read_u32::<LittleEndian>()
                            .map_err(|_| "fail to read expire")?
                            * 1000) as u128,
                    );
                }
                OpCode::EXPIRETIMEMS => {
                    let timestamp = self.read_bytes(8)?;
                    let mut rdr = Cursor::new(timestamp);
                    expires_at = Some(
                        rdr.read_u64::<LittleEndian>()
                            .map_err(|_| "fail to read expire")? as u128,
                    );
                }
                _ => {
                    let value_type = next_op;
                    let key = self.read_string()?;
                    let value = self.read_value(value_type)?;
                    let val_str = value.to_string();
                    (self.handler)(db_index, key.to_string(), val_str, expires_at);

                    expires_at = None;
                }
            }
        }
        Ok(rdb)
    }

    fn parse_header(&mut self) -> Result<RdbHeader, String> {
        let magic_bytes = self.read_bytes(5)?;
        let magic =
            String::from_utf8(magic_bytes.to_vec()).map_err(|_| "Invalid magic".to_string())?;

        let version_bytes = self.read_bytes(4)?;
        let version =
            String::from_utf8(version_bytes.to_vec()).map_err(|_| "Invalid version".to_string())?;
        Ok(RdbHeader { magic, version })
    }
    fn read_bytes(&mut self, length: usize) -> Result<&[u8], String> {
        if self.pos + length > self.input.len() {
            return Err("length exceeds input".to_string());
        }
        let start = self.pos;
        self.pos += length;
        Ok(&self.input[start..self.pos])
    }
    fn read_byte(&mut self) -> Result<u8, String> {
        let byte = self.read_bytes(1)?[0];
        Ok(byte)
    }
    fn read_length(&mut self) -> Result<RdLength, String> {
        let byte = self.read_byte()?;
        match byte >> 6 {
            0b00 => {
                let result = (byte & 0b0011_1111) as u32;
                Ok(RdLength::Len(result))
            }
            0b01 => {
                let next_byte = self.read_byte()? as u32;
                let rest = (byte & 0b1100_0000) as u32;
                let result = (rest << 8) | next_byte;
                Ok(RdLength::Len(result))
            }
            0b10 => {
                let next_bytes = self.read_bytes(4)?;
                let result = u32::from_be_bytes(next_bytes.try_into().unwrap());
                Ok(RdLength::Len(result))
            }
            0b11 => {
                let format = byte & 0b0011_1111;
                match format {
                    0 => Ok(RdLength::Integer(1)),
                    1 => Ok(RdLength::Integer(2)),
                    2 => Ok(RdLength::Integer(4)),
                    3 => Ok(RdLength::LZF),
                    _ => Err("Invalid length format".to_string()),
                }
            }
            4_u8..=u8::MAX => unreachable!(),
        }
    }
    fn read_string(&mut self) -> Result<RdbString, String> {
        match self.read_length()? {
            RdLength::Len(length) => {
                let value = self.read_bytes(length as usize)?;
                Ok(RdbString::String(value.to_vec()))
            }
            RdLength::Integer(length) => {
                let value = self.read_bytes(length as usize)?;
                Ok(RdbString::Integer(value.to_vec()))
            }
            RdLength::LZF => Ok(RdbString::LZF(vec![])),
        }
    }
    fn read_value(&mut self, value_type: u8) -> Result<RdbValue, String> {
        match RdValueType::from(value_type) {
            RdValueType::String => {
                let value = self.read_string()?;
                Ok(RdbValue::String(value))
            }
            _ => Err("Invalid value type".to_string()),
        }
    }
}

#[derive(Debug)]
pub struct Rdb {
    pub header: RdbHeader,
    pub metadata: RdbMetadata,
    // pub databases: Vec<RdbDatabase>,
    pub checksum: u64,
}

#[derive(Debug)]
pub struct RdbHeader {
    pub magic: String,
    pub version: String,
}

#[derive(Debug)]
pub struct RdbMetadata {
    info: HashMap<String, String>,
}

// #[derive(Debug)]
// pub struct RdbDatabase {
//     pub db_index: usize,
//     pub entries: Vec<Entry>,
// }

// #[derive(Debug)]
// pub struct Entry {
//     pub key: String,
//     pub value: RdbValue,
//     pub expired: Option<u64>,
// }

#[derive(Debug)]
pub enum RdbValue {
    /// 字符串类型
    String(RdbString),
    /// 列表类型
    List(Vec<String>),
    /// 集合类型
    Set(Vec<String>),
    /// 有序集合类型
    SortedSet(Vec<SortedSetEntry>),
    /// 哈希类型
    Hash(HashMap<String, String>),
}

impl ToString for RdbValue {
    fn to_string(&self) -> String {
        match self {
            RdbValue::String(s) => s.to_string(),
            RdbValue::List(l) => l.join(","),
            RdbValue::Set(s) => s.join(","),
            RdbValue::SortedSet(s) => s
                .iter()
                .map(|e| format!("{}:{}", e.member, e.score))
                .collect::<Vec<String>>()
                .join(","),
            RdbValue::Hash(h) => h
                .iter()
                .map(|(k, v)| format!("{}:{}", k, v))
                .collect::<Vec<String>>()
                .join(","),
        }
    }
}

#[derive(Debug)]
pub struct SortedSetEntry {
    /// 成员
    pub member: String,
    /// 分数
    pub score: f64,
}
