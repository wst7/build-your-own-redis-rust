use std::{collections::HashMap, sync::LazyLock};
use time::OffsetDateTime;
use tokio::{sync::RwLock, time::Instant};
use crate::rdb::RdbValue;

static STORAGE: LazyLock<RwLock<HashMap<String, Item>>> =
    LazyLock::new(|| RwLock::new(HashMap::new()));

#[derive(Clone, Debug)]
struct Item {
    value: String,
    expires: Option<OffsetDateTime>,
    created: Instant,
}

pub async fn set(key: &str, value: &str, expires: Option<OffsetDateTime>) {
    let mut store = STORAGE.write().await;
    let item = Item {
        value: value.to_string(),
        expires: expires,
        created: Instant::now(),
    };
    store.insert(key.to_string(), item);
}

pub async fn get(key: &str) -> Option<String> {
    let store = STORAGE.read().await;
    let item = store.get(key).cloned();
    match item {
        Some(item) => {
            if let Some(expires) = item.expires {
                if expires < OffsetDateTime::now_utc(){
                    // let mut store = STORAGE.write().await;
                    // store.remove(key);
                    // TODO: Remove the key from the storage
                    return None;
                }
            }
            Some(item.value)
        }
        None => None,
    }
}

pub async fn keys() -> Vec<String> {
    let store = STORAGE.read().await;
    store.keys().cloned().collect()
}
