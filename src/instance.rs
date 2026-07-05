use std::{collections::HashMap, rc::Rc};

use crate::{class::Class, token::Token, value::Value};

#[derive(Debug)]
pub struct Instance {
    pub class: Rc<Class>,
    fields: HashMap<String, Value>,
}

impl Instance {
    pub fn new(class: Rc<Class>) -> Self {
        Instance {
            class,
            fields: HashMap::new(),
        }
    }

    pub fn get(&self, name: &Token) -> Option<Value> {
        self.fields
            .get(&name.lexeme)
            .or_else(|| self.class.find_method(&name.lexeme))
            .cloned()
    }

    pub fn set(&mut self, name: &Token, value: &Value) {
        self.fields.insert(name.lexeme.clone(), value.clone());
    }
}
