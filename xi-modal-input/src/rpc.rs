
use serde_json::Value;
use serde::de::DeserializeOwned;

#[derive(Debug, Clone, Deserialize)]
pub struct Rpc<'a> {
    pub method: &'a str,
    pub params: Value,
}

impl<'a> Rpc<'a> {
    pub fn get_params<T: DeserializeOwned>(self) -> T {
        serde_json::from_value(self.params).unwrap()
    }
}
