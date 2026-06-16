use crate::value::Value;
use std::collections::hash_map::Entry;
use std::{cell::RefCell, collections::HashMap, rc::Rc};

#[derive(Debug)]
pub struct Environment {
    values: HashMap<String, Value>,
    enclosing: Option<Rc<RefCell<Environment>>>,
}

impl Environment {
    pub fn new(enclosing: Option<Rc<RefCell<Environment>>>) -> Self {
        Environment {
            values: HashMap::new(),
            enclosing,
        }
    }

    pub fn define(&mut self, name: String, value: Value) {
        self.values.insert(name, value);
    }

    pub fn get(&self, name: &str) -> Result<Value, String> {
        match self.values.get(name) {
            Some(v) => Ok(v.to_owned()),
            None => match &self.enclosing {
                Some(e) => e.borrow().get(name),
                None => Err(format!("Undefined variable: '{}'.", name)),
            },
        }
    }

    pub fn assign(&mut self, name: String, value: Value) -> Result<(), String> {
        match self.values.entry(name) {
            Entry::Occupied(mut e) => {
                e.insert(value);
                Ok(())
            }
            Entry::Vacant(e) => {
                let name = e.into_key();
                match &self.enclosing {
                    Some(env) => env.borrow_mut().assign(name, value),
                    None => Err(format!("Undefined variable: '{}'.", name)),
                }
            }
        }
    }
}
