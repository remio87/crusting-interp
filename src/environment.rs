use crate::value::Value;
use std::collections::HashMap;

#[derive(Debug)]
pub struct Environment {
    values: HashMap<String, Value>,
}

impl Environment {
    pub fn new() -> Self {
        Environment {
            values: HashMap::new(),
        }
    }

    pub fn define(&mut self, name: String, value: Value) {
        self.values.insert(name, value);
    }

    pub fn get(&self, name: &str) -> Result<&Value, String> {
        match self.values.get(name) {
            Some(v) => Ok(v),
            None => Err(format!("Undefined variable: '{}'.", name)),
        }
    }
}
