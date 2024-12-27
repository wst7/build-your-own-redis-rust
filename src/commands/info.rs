use crate::resp::RespType;


pub async fn info() -> Result<RespType, String> {
    Ok(RespType::SimpleString("role:master".to_string()))
}