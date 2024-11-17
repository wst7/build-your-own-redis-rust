use std::{collections::HashMap, sync::LazyLock};

use tokio::sync::RwLock;

static STORE: LazyLock<RwLock<HashMap<String, String>>> = LazyLock::new(|| RwLock::new(HashMap::new()));

pub async fn set(key: &str, value: &str) {
  let mut store = STORE.write().await;
  store.insert(key.to_string(), value.to_string());
}

pub async fn get(key: &str) -> Option<String> {
  let store = STORE.read().await;
  store.get(key).cloned()
}