use crate::resp::RespType;

pub fn echo(args: Vec<String>) -> Result<RespType, String> {
    Ok(RespType::SimpleString(args[0].clone()))
}
