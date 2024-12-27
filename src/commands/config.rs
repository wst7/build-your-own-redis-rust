use crate::{config, resp::RespType};

// pub async fn config_set(args: Vec<String>) {
//     let mut config = CONFIG.lock().await;
//     config.insert(key.to_string(), value.to_string());
// }

pub async fn config_get(args: Vec<String>) -> Result<RespType, String> {
    let parameter = &args[1];
    let value = config::get(&parameter).await.unwrap();
    let vec = vec![RespType::BulkString(Some(parameter.to_string())), RespType::BulkString(Some(value))];
    Ok(RespType::Array(Some(vec)))
}
