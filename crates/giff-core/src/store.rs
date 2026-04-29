use crate::{GiffError, StackStore};

impl StackStore {
    pub fn from_toml(s: &str) -> Result<Self, GiffError> {
        toml::from_str(s).map_err(|e| GiffError::Parse(e.to_string()))
    }

    pub fn to_toml(&self) -> Result<String, GiffError> {
        toml::to_string_pretty(self).map_err(|e| GiffError::Parse(e.to_string()))
    }
}
