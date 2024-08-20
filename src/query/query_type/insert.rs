use super::KeyVal;

#[derive(Debug)]
pub struct InsertQuery {
    pub key_vals: Vec<KeyVal>,
}

impl InsertQuery {
    /// Returns the value of the key if it exists
    pub fn get_key_val(&self, key: &str) -> Option<&str> {
        self.key_vals.iter().find_map(|key_val| {
            if key_val.key == key {
                Some(key_val.val.as_str())
            } else {
                None
            }
        }) 
    }

    /// Returns owned keys
    pub fn get_keys(&self) -> Vec<String> {
        self.key_vals.iter().map(|key_val| key_val.key.clone()).collect()
    }
}
