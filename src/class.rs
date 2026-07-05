use std::collections::HashMap;

use crate::value::Value;

#[derive(Debug)]
pub struct Class {
    pub name: String,
    methods: HashMap<String, Value>,
}

impl std::fmt::Display for Class {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}

impl Class {
    pub fn new(name: &str, methods: HashMap<String, Value>) -> Self {
        Class {
            name: name.to_string(),
            methods,
        }
    }

    pub fn find_method(&self, name: &str) -> Option<&Value> {
        self.methods.get(name)
    }
}
