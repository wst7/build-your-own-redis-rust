use crate::{config, resp::RespType};




pub async fn info() -> Result<RespType, String> {
    let role = match config::get("replicaof").await {
        Some(_) => "slave",
        None => "master",
    };
    Ok(RespType::SimpleString(format!("role: {}", role)))
}