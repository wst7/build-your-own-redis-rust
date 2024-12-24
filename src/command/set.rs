use crate::storage;


pub async fn set(args: Vec<String>) -> Result<String, String> {
  let mut expires = None;
  if args.len() == 4 && &args[2].to_uppercase() == "PX" {
      expires = Some(args[3].parse::<u128>().unwrap());
  }
  storage::set(&args[0], &args[1], expires).await;
  Ok(format!("+OK\r\n"))
}