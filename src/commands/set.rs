use time::{Duration, OffsetDateTime};

use crate::{resp::RespType, storage};

pub async fn set(args: Vec<String>) -> Result<RespType, String> {
    let mut expires = None;
    if args.len() == 4  {
        if &args[2].to_uppercase() == "PX" {
            expires = Some(OffsetDateTime::now_utc() + Duration::milliseconds(args[3].parse::<i64>().unwrap()));
        }
        if &args[2].to_uppercase() == "EX" {
            expires = Some(OffsetDateTime::now_utc() + Duration::seconds(args[3].parse::<i64>().unwrap()));
        }
    }
    storage::set(&args[0], &args[1], expires).await;
    Ok(RespType::SimpleString("OK".to_string()))
}
