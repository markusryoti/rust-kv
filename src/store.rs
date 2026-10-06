use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

#[derive(Debug)]
pub struct Store {
    store: Arc<Mutex<HashMap<String, String>>>,
}

#[derive(Debug)]
pub enum StoreError {
    MutexError,
}

impl Store {
    pub fn new() -> Self {
        Store {
            store: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn get(&self, key: &str) -> Result<String, StoreError> {
        match self.store.lock() {
            Ok(store) => {
                let v = store.get(key).unwrap_or(&"".to_string()).clone();
                Ok(v)
            }
            Err(_) => Err(StoreError::MutexError),
        }
    }

    pub fn set(&self, key: String, value: String) -> Result<String, StoreError> {
        match self.store.lock() {
            Ok(mut store) => {
                store.insert(key, value.clone());
                Ok(value)
            }
            Err(_) => Err(StoreError::MutexError),
        }
    }
}

impl Default for Store {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn set_and_get() {
        let store = Store::new();
        store
            .set(String::from("key"), String::from("value"))
            .unwrap();
        let value = store.get("key").unwrap();
        assert_eq!(value, "value".to_string());
    }

    #[test]
    fn get_empty() {
        let store = Store::new();
        let value = store.get("key").unwrap();
        assert_eq!(value, "");
    }
}
