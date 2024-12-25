use std::{collections::HashMap, sync::LazyLock};

use tokio::sync::Mutex;

static CONFIG: LazyLock<Mutex<HashMap<String, String>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

pub async fn set(key: &str, value: &str) {
    let mut config = CONFIG.lock().await;
    config.insert(key.to_string(), value.to_string());
}
pub async fn get(key: &str) -> Option<String> {
    let config = CONFIG.lock().await;
    config.get(key).cloned()
}
