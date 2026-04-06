use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::interpreter::Value;

pub struct Environment {
    parent: Option<Rc<RefCell<Self>>>,
    values: HashMap<String, Value>,
}

impl Environment {
    #[must_use]
    pub fn new() -> Self {
        Self {
            parent: None,
            values: HashMap::new(),
        }
    }

    #[must_use]
    pub fn new_enclosed(parent: Rc<RefCell<Self>>) -> Self {
        Self {
            parent: Some(parent),
            values: HashMap::new(),
        }
    }

    pub fn define(&mut self, name: String, value: Value) {
        self.values.insert(name, value);
    }

    pub fn assign(&mut self, name: String, value: Value) -> bool {
        if let Some(slot) = self.values.get_mut(&name) {
            *slot = value;
            return true;
        }

        if let Some(parent) = &mut self.parent {
            return parent.borrow_mut().assign(name, value);
        }

        false
    }

    #[must_use]
    pub fn get(&self, name: &str) -> Option<Value> {
        if let Some(value) = self.values.get(name) {
            return Some(value.clone());
        }

        if let Some(parent) = &self.parent {
            return parent.borrow().get(name);
        }

        None
    }

    #[must_use]
    pub fn contains(&self, name: &str) -> bool {
        self.values.contains_key(name)
            || self
                .parent
                .as_ref()
                .is_some_and(|parent| parent.borrow().contains(name))
    }
}
