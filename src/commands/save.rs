use crate::resp::RespType;

pub async fn save() -> Result<RespType, String> {
    Ok(RespType::SimpleString("OK".to_string()))
}
