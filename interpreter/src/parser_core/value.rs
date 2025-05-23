//**NOTE** This functionality is seperated from the runtime value as it includes parsing intermediary types

#[derive(Debug, Clone)]
pub enum Value {
    Int(i32),
    Str(String),
    Char(char),
    Float(f64),
    VarName(String),
    Undefined,
}


// **NOTE** For each type there must be 3 associated functions: 
//    1. an implementation in the evaluate type from value method, (Implemented as a function shared across all Values)
//    2. an implementation in the evaluate from string method, (Implemented as a function shared across all Values)
//    3. a cast to function and implementation in other cast to's, (Implemented as traits of each Value type)

// Trait for casting Value to specific types
pub trait CastTo<T> {
    fn cast_to(&self) -> Option<Value>;
}

// Implementation for casting to i32
impl CastTo<i32> for Value {
    fn cast_to(&self) -> Option<Value> {
        match self {
            Value::Int(n) => Some(Value::Int(*n)),
            Value::Str(s) => s.parse().ok().map(Value::Int),
            Value::Char(c) => Some(Value::Int(*c as i32)),
            Value::Float(f) => Some(Value::Int(*f as i32)),
            Value::VarName(_) => None,
            Value::Undefined => None,
        }
    }
}

// Implementation for casting to f64
impl CastTo<f64> for Value {
    fn cast_to(&self) -> Option<Value> {
        match self {
            Value::Int(n) => Some(Value::Float(*n as f64)),
            Value::Str(s) => s.parse().ok().map(Value::Float),
            Value::Char(c) => Some(Value::Float(*c as i32 as f64)),
            Value::Float(f) => Some(Value::Float(*f)),
            Value::VarName(_) => None,
            Value::Undefined => None,
        }
    }
}

// Implementation for casting to String
impl CastTo<String> for Value {
    fn cast_to(&self) -> Option<Value> {
        match self {
            Value::Int(n) => Some(Value::Str(n.to_string())),
            Value::Str(s) => Some(Value::Str(s.clone())),
            Value::Char(c) => Some(Value::Str(c.to_string())),
            Value::Float(f) => Some(Value::Str(f.to_string())),
            Value::VarName(v) => Some(Value::Str(v.clone())),
            Value::Undefined => None,
        }
    }
}

// Implementation for casting to char
impl CastTo<char> for Value {
    fn cast_to(&self) -> Option<Value> {
        match self {
            Value::Int(n) => char::from_u32(*n as u32).map(Value::Char),
            Value::Str(s) => {
                if s.len() == 1 {
                    s.chars().next().map(Value::Char)
                } else {
                    None
                }
            },
            Value::Char(c) => Some(Value::Char(*c)),
            Value::Float(f) => char::from_u32(*f as u32).map(Value::Char),
            Value::VarName(_) => None,
            Value::Undefined => None,
        }
    }
}

// Implementation for casting to Undefined
impl CastTo<()> for Value {
    fn cast_to(&self) -> Option<Value> {
        Some(Value::Undefined)
    }
}

impl Value {
    pub fn evaluate(val: String) -> Value {   // Converts string representation of type to Value
        // Check if the value is wrapped in quotes (string)
        if val.starts_with("\"") && val.ends_with("\"") {
            // Remove the quotes and return as string
            Value::Str(val[1..val.len()-1].to_string())
        } else if val.starts_with("\'") && val.ends_with("\'") {
            // Handle character value
            let char_str = &val[1..val.len()-1];
            if char_str.len() == 1 {
                Value::Char(char_str.chars().next().unwrap())
            } else {
                Value::Str(char_str.to_string()) // If not a single character, treat as string
            }
        } else {
            // First try to parse as float
            if let Ok(float_val) = val.parse::<f64>() {
                // If it's a whole number, return as Int
                if float_val == float_val.trunc() {
                    Value::Int(float_val as i32)
                } else {
                    Value::Float(float_val)
                }
            } else {
                // If not a float, try as integer
                match val.parse::<i32>() {
                    Ok(int_val) => Value::Int(int_val),
                    Err(_) => {
                        // If it's not a number and not wrapped in quotes, treat as variable name
                        if val.chars().all(|c| c.is_alphanumeric() || c == '_') {
                            Value::VarName(val)
                        } else {
                            Value::Undefined
                        }
                    }
                }
            }
        }
    }

    pub fn get_type(val: Value) -> Value {
        match val {
            Value::Int(_) => Value::Str("int".to_string()),
            Value::Str(_) => Value::Str("str".to_string()),
            Value::Char(_) => Value::Str("char".to_string()),
            Value::Float(_) => Value::Str("float".to_string()),
            Value::VarName(_) => Value::Str("var".to_string()),
            Value::Undefined => Value::Str("undefined".to_string()),
        }
    }
}