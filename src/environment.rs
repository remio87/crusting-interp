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

    fn ancestor(
        env: Rc<RefCell<Environment>>,
        distance: usize,
    ) -> Result<Rc<RefCell<Environment>>, String> {
        let mut current = env;
        for _ in 0..distance {
            let next = match &current.borrow().enclosing {
                Some(e) => Rc::clone(e),
                None => return Err("Environment depth is too shallow.".to_string()),
            };
            current = next;
        }
        Ok(current)
    }

    pub fn get_at(
        env: Rc<RefCell<Environment>>,
        distance: usize,
        name: &str,
    ) -> Result<Value, String> {
        match Environment::ancestor(env, distance)?
            .borrow()
            .values
            .get(name)
        {
            Some(v) => Ok(v.clone()),
            None => Err(format!("Undefined variable: '{name}'.")),
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

    pub fn assign_at(
        env: Rc<RefCell<Environment>>,
        distance: usize,
        name: String,
        value: Value,
    ) -> Result<(), String> {
        match Environment::ancestor(env, distance)?
            .borrow_mut()
            .values
            .entry(name)
        {
            Entry::Occupied(mut e) => {
                e.insert(value);
                Ok(())
            }
            Entry::Vacant(e) => {
                let name = e.into_key();
                Err(format!(
                    "Undefined variable in the specified depth. Variable: '{name}'"
                ))
            }
        }
    }
}
