use crate::{resp::RespType, storage};
use regex::Regex;

pub async fn get(args: Vec<String>) -> Result<RespType, String> {
    Ok(match storage::get(&args[0]).await {
        Some(value) => RespType::SimpleString(value),
        None => RespType::BulkString(None),
    })
}

pub async fn keys(args: Vec<String>) -> Result<RespType, String> {
    if args.len() != 1 {
        return Ok(RespType::SimpleError("wrong number of arguments for 'keys' command".to_string()));
    }

    let regex_pattern = &args[0].replace('*', ".*"); // Replace '*' with '.*' (wildcard)

    let patten = Regex::new(regex_pattern).unwrap();
    let keys = storage::keys()
        .await
        .iter()
        .filter(|key| patten.is_match(key))
        .map(|key| key.to_string())
        .collect::<Vec<String>>();
    
    let values = keys
        .iter()
        .map(|key| RespType::SimpleString(key.to_string()))
        .collect::<Vec<RespType>>();
    let reply = RespType::Array(Some(values));
    Ok(reply)
}
