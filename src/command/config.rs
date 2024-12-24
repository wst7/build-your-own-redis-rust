use crate::config;


// pub async fn config_set(args: Vec<String>) {
//     let mut config = CONFIG.lock().await;
//     config.insert(key.to_string(), value.to_string());
// }

pub async fn config_get(args: Vec<String>) -> Result<String, String> {
  let parameter = &args[1];
  let parameter_len = parameter.len();
  let value = config::get(&parameter).await.unwrap();
  let len = value.len();
  Ok(format!(
      "*2\r\n${}\r\n{}\r\n${}\r\n{}\r\n",
      parameter_len, parameter, len, value
  ))
}