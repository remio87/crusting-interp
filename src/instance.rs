use std::{cell::RefCell, collections::HashMap, rc::Rc};

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

    pub fn get(instance: Rc<RefCell<Instance>>, name: &Token) -> Option<Value> {
        if let Some(field) = instance.borrow().fields.get(&name.lexeme) {
            Some(field.clone())
        } else if let Some(method) = instance.borrow().class.find_method(&name.lexeme) {
            Some(method.bind(Rc::clone(&instance)))
        } else {
            None
        }
    }

    pub fn set(&mut self, name: &Token, value: &Value) {
        self.fields.insert(name.lexeme.clone(), value.clone());
    }
}
