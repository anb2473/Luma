use std::collections::HashMap;
use crate::parser_core::value::Value;
use std::rc::{Rc};

pub struct Environment {
    pub vars: HashMap<String, Value>,    // Var name to Value (from enum) to allow fast look up times
    pub parent: Option<Rc<Environment>>, // Smart pointer back to parent environment to allow nested environments
}

impl Environment {
    pub fn new(parent: Option<Rc<Environment>>) -> Self {
        // Generate a new clean environment
        Environment {
            vars: HashMap::new(),
            parent: parent,
        }
    }

    pub fn search_for_var(&self, name: String) -> Value {
        // First check if the value exists in the current environment
        if let Some(val) = self.vars.get(&name) {
            return val.clone();  // Return a clone of the value
        }
        
        // If not found, check parent environments
        if let Some(parent_env) = &self.parent {
            return parent_env.search_for_var(name);
        }
        
        // If no value is found in any environment, return Null
        Value::Undefined
    }
}