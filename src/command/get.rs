use crate::storage;
use regex::Regex;

pub async fn get(args: Vec<String>) -> Result<String, String> {
    Ok(match storage::get(&args[0]).await {
        Some(value) => format!("+{}\r\n", value),
        None => format!("$-1\r\n"),
    })
}

pub async fn keys(args: Vec<String>) -> Result<String, String> {
    if args.len() != 1 {
        return Err("-ERR wrong number of arguments for 'keys' command\r\n".to_string());
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
        .map(|key| format!("+{}\r\n", key))
        .collect::<Vec<String>>()
        .join("");
    let reply = format!("*{}\r\n{}", keys.len(), values);
    Ok(reply)
}
