use crate::parser_core::value::{Value};

struct Environment {
    vars: HashMap<Value, Value>,
    parent: Rc<Environment>,
}

impl Environment {
    fn new(parent: Option<Rc<Environment>>) -> Self {
        // Generate a new clean environment
        Environment {
            vars: HashMap::new(),
            parent: parent,
            dependencies: HashMap::new(),
        }
    }

    fn search_for_var(&self, name: String) -> Value {
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