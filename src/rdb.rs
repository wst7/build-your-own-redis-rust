use std::{collections::HashMap, io::Read, path::PathBuf, vec};

enum OpCode {
  AUX = 0xFA,
  RESIZEDB = 0xFB,
  EXPIRETIMEMS = 0xFC,
  EXPIRETIME = 0xFD,
  SELECTDB = 0xFE,
  EOF = 0xFF,
  Unknown,
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
pub struct RdbParser<R: Read> {
    input: R,
}
impl<R: Read> RdbParser<R> {
    pub fn new(input: R) -> Self {
        Self { input }
    }
    pub fn parse(&mut self) -> Result<Rdb, String> {
        let header = self.parse_header()?;
        let mut rdb = Rdb {
            header,
            meta: vec![],
            datas: vec![],
            checksum: 0,
        };
        let mut next_op = self.read_byte();
        loop {
            match OpCode::from(next_op) {
                OpCode::AUX => {}
                OpCode::EOF => {
                    let mut checksum = vec![];
                    self.input.read_to_end(&mut checksum).unwrap();
                    rdb.checksum = u64::from_le_bytes(checksum.try_into().unwrap());
                    break;
                }
                OpCode::SELECTDB => {}
                OpCode::RESIZEDB => {}
                OpCode::EXPIRETIME => {}
                OpCode::EXPIRETIMEMS => {}
                _ => {}
            }
        }
        rdb
    }
    fn read_byte(&mut self) -> u8 {
        let mut buf = [0u8; 1];
        self.input.read_exact(&mut buf).unwrap();
        buf[0]
    }
    fn parse_header(&mut self) -> Result<RdbHeader, String> {
        let mut magic_bytes = [0u8; 5];
        self.input.read_exact(&mut magic_bytes).unwrap();
        let magic = String::from_utf8(magic_bytes.to_vec()).unwrap();
        let mut version_bytes = [0u8; 4];
        self.input.read_exact(&mut version_bytes).unwrap();
        let version = String::from_utf8_lossy(&version_bytes).parse::<u32>()?;
       Ok( RdbHeader { magic, version })
    }
}

pub struct Rdb {
    pub header: RdbHeader,
    pub meta: Vec<RdbMeta>,
    pub datas: Vec<RdbDatabase>,
    pub checksum: u64,
}

pub struct RdbHeader {
    pub magic: String,
    pub version: u32,
}

pub struct RdbMeta {
    pub offset: u64,
    pub length: u64,
}

pub struct RdbDatabase {
    pub expired_hash_table: HashMap<String, RdbExpireValue>,
    pub hash_table: HashMap<String, String>,
}

pub struct RdbExpireValue {
    pub value: String,
    pub expired: u64,
}
