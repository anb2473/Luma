use std::collections::HashMap;
use std::rc::Rc;
use std::path::{Path};
use std::fs::{File};
use std::{io};
use std::io::{Read, Write};
use std::fmt::Debug;

// We need to integratea  custom implementation of Clone for the fn (Vec) for the Value to work properly
// Trait that is both Fn and Clone
pub trait CloneFn: Fn(Vec<Value>) -> Value + 'static {
    fn clone_box(&self) -> Box<dyn CloneFn>;
}

// Blanket implementation for all matching types
impl<T> CloneFn for T
where
    T: Fn(Vec<Value>) -> Value + Clone + 'static,
{
    fn clone_box(&self) -> Box<dyn CloneFn> {
        Box::new(self.clone())
    }
}

// Implement Clone manually for Box<dyn CloneFn>
impl Clone for Box<dyn CloneFn> {
    fn clone(&self) -> Box<dyn CloneFn> {
        self.clone_box()
    }
}

// File handling functions

fn read_file(file_path: &str) -> Result<String, io::Error> {
    let path = Path::new(file_path);
    let mut file = File::open(path)?; // Open the file, propagate errors

    let mut contents = String::new();
    file.read_to_string(&mut contents)?; // Read contents, propagate errors

    Ok(contents) // Return the contents if successful
}

fn write_file(file_path: &str, contents: &str) -> Result<(), io::Error> {
    let path = Path::new(file_path);
    let mut file = File::create(path)?; // Create or truncate the file, propagate errors

    file.write_all(contents.as_bytes())?; // Write the contents, propagate errors

    Ok(()) // Return Ok if successful
}

// Abstract syntax tree (AST): on compilation all the lines in the functions will be compiled to the AST format before execution
// AST is a Vec<Vec<String>, Which contains a Vec of lines, with a sub Vec of the parts of the line
// Each line will be split into a verb and nouns

#[derive(Clone, Debug)]
enum Verb {
    Set,     // x = 0;
    Return,  // x
    Mark,  // my_import!
    Do,
}

#[derive(Clone, Debug)]
struct ASTLine {
    pub verb: Verb,
    pub nouns: Vec<String>,
}

#[derive(Clone, Debug)]
struct AST {
    pub lines: Vec<ASTLine>,
    pub params: HashMap<String, String>,
}

// Value enum will hold all the referencable types

#[derive(Clone)]
enum Value {
    Int(i32),
    Float(f64),
    Str(String),
    Bool(bool),
    List(Vec<Value>),
    FunctionRef(Box<dyn CloneFn>),   // Holds a function reference (This will allow for you to call Rust functions) **NOTE:** Function takes in vec
    ASTRef(AST), // Holds a function reference (This will allow for you to call Luma functions)
    Environment(Rc<Environment>),   // Holds a smart pointer to an environment (for classes)
    Undefined,
    Null,
}

impl PartialEq for Value {
    fn eq(&self, other: &Value) -> bool {
        match (self, other) {
            (Value::Int(a), Value::Int(b)) => a == b,
            (Value::Float(a), Value::Float(b)) => a == b,
            (Value::Str(a), Value::Str(b)) => a == b,
            (Value::Bool(a), Value::Bool(b)) => a == b,
            (Value::List(a), Value::List(b)) => a == b,
            (Value::Null, Value::Null) => true,
            (Value::Undefined, Value::Undefined) => true,
            // Allow comparing int with float
            (Value::Int(a), Value::Float(b)) => *a as f64 == *b,
            (Value::Float(a), Value::Int(b)) => *a == *b as f64,
            _ => false,
        }
    }
}

impl PartialOrd for Value {
    fn partial_cmp(&self, other: &Value) -> Option<std::cmp::Ordering> {
        match (self, other) {
            (Value::Int(a), Value::Int(b)) => a.partial_cmp(b),
            (Value::Float(a), Value::Float(b)) => a.partial_cmp(b),
            (Value::Str(a), Value::Str(b)) => a.partial_cmp(b),
            (Value::Bool(a), Value::Bool(b)) => a.partial_cmp(b),
            // Allow comparing int with float
            (Value::Int(a), Value::Float(b)) => (*a as f64).partial_cmp(b),
            (Value::Float(a), Value::Int(b)) => a.partial_cmp(&(*b as f64)),
            // Lists can be compared if they contain comparable values
            (Value::List(a), Value::List(b)) => {
                if a.len() != b.len() {
                    return a.len().partial_cmp(&b.len());
                }
                for (x, y) in a.iter().zip(b.iter()) {
                    if let Some(ord) = x.partial_cmp(y) {
                        if ord != std::cmp::Ordering::Equal {
                            return Some(ord);
                        }
                    }
                }
                Some(std::cmp::Ordering::Equal)
            },
            _ => None,
        }
    }
}

// For converting raw string nouns to actual values

trait ToValue {
    fn to_value(Value) -> Value;
}

impl Value {
    // Function implementation to call casting function between types

    pub fn cast_to<T: ToValue>(self) -> Value {
        T::to_value(self)
    }

    // Functions to generate a value from a str representation

    // Orginization function

    // Parse a string into a Value::Int (i32)
    pub fn parse_int(input: &str) -> Option<Value> {
        match input.parse::<i32>() {
            Ok(n) => Some(Value::Int(n)),
            Err(_) => None, // If parsing fails, return None
        }
    }

    // Parse a string into a Value::Float (f64)
    pub fn parse_float(input: &str) -> Option<Value> {
        match input.parse::<f64>() {
            Ok(f) => Some(Value::Float(f)),
            Err(_) => None,
        }
    }

    // Parse a string into a Value::Str
    pub fn parse_str(input: &str) -> Option<Value> {
        Some(Value::Str(input.to_string()))
    }

    // Parse a string into a Value::Bool
    pub fn parse_bool(input: &str) -> Option<Value> {
        match input.to_lowercase().as_str() {
            "true" => Some(Value::Bool(true)),
            "false" => Some(Value::Bool(false)),
            _ => None,
        }
    }

    pub fn parse(input: &str, type_of: &str) -> Value {
        match type_of {
            "int" => Self::parse_int(input),
            "str" => Self::parse_str(input),
            "float" => Self::parse_float(input),
            "bool" => Self::parse_bool(input),
            _ => {
                panic!("Unknown type {}", type_of);
            }
        };

        Value::Null
    }

    pub fn print_value(&self) -> String {
        match self {
            Value::Int(i) => i.to_string(),
            Value::Float(f) => f.to_string(),
            Value::Str(s) => s.clone(),
            Value::Bool(b) => b.to_string(),
            Value::List(l) => {
                let mut result = String::from("[");
                for (i, v) in l.iter().enumerate() {
                    if i > 0 {
                        result.push_str(", ");
                    }
                    result.push_str(&v.print_value());
                }
                result.push(']');
                result
            },
            Value::FunctionRef(_) => String::from("<function>"),
            Value::ASTRef(_) => String::from("<ast>"),
            Value::Environment(_) => String::from("<environment>"),
            Value::Undefined => String::from("undefined"),
            Value::Null => String::from("null"),
        }
    }

    pub fn print_type(&self) -> String {
        match self {
            Value::Int(_) => String::from("int"),
            Value::Float(_) => String::from("float"),
            Value::Str(_) => String::from("string"),
            Value::Bool(_) => String::from("bool"),
            Value::List(_) => String::from("list"),
            Value::FunctionRef(_) => String::from("function"),
            Value::ASTRef(_) => String::from("ast"),
            Value::Environment(_) => String::from("environment"),
            Value::Undefined => String::from("undefined"),
            Value::Null => String::from("null"),
        }
    }
}


// **NOTE:** When a request for a value in the environment is made the system will start by searching through the raw environment,
// Before following the pointers to each parent environment and searching for the value there

#[derive(Clone)] // Derive Clone if you want to clone instances of Environment
struct Environment {
    vars: HashMap<String, Value>,    // Var name to Value (from enum) to allow fast look up times
    parent: Option<Rc<Environment>>, // Smart pointer back to parent environment to allow nested environments
}

impl Environment {
    fn new() -> Self {
        // Generate a new clean environment
        Environment {
            vars: HashMap::new(),
            parent: None,
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

struct ASTGenerator {
    file_contents: String,
    environment: Environment,
}

impl ASTGenerator {
    fn new(path: &str, environment: Environment) -> Self {
        let file_contents = match read_file(path) {
            Ok(val) => val,
            Err(_) => panic!("File not found"),
        };

        ASTGenerator {
            file_contents: file_contents,
            environment: environment,
        }
    }

    // Self must be mutable as we mutate the Environment attribute
    fn fn_to_AST(function: String, type_of: String, params: HashMap<String, String>, function_name: String) -> (String, Value) {
        let mut lines = function.split("\n");

        let mut next_line = lines.next();

        let mut ast = AST { lines: Vec::new(), params };

        while let Some(line) = next_line {
            if line == "" {
                next_line = lines.next();
                continue;
            }
            let simplified_line = &line[0..line.len() - 1]; // Remove suffix

            let suffix = match line.chars().last() {
                Some(val) => val,
                None => panic!("Failed to read suffix")
            };
            
            match suffix {
                '?' => {    // Do statement (syntax: marker_name conditional?)
                    let mut split_line = simplified_line.splitn(2, " ");

                    ast.lines.push(ASTLine {
                        verb: Verb::Do,
                        nouns: vec![match split_line.next() {
                            Some(val) => val.to_string(),
                            None => panic!("Failed to load marker name of do statement")
                        }, match split_line.next() {
                            Some(val) => val.to_string(),
                            None => panic!("Failed to load conditional statement of do statement")
                        }],
                    })
                },
                ';' => {    // Set statement (syntax: var: type = val;)
                    let mut parts = simplified_line.split("=");

                    let var_declaration = match parts.next() {
                        Some(val) => val,
                        None => {
                            panic!("Failed to load first half of declaration")
                        }
                    };

                    let val = match parts.next() {
                        Some(val) => val,
                        None => {
                            panic!("Failed to load variable value")
                        }
                    }.trim();

                    let mut declaration_parts = var_declaration.split(":");

                    let var_name = match declaration_parts.next() {
                        Some(val) => val.trim(),
                        None => {
                            panic!("Failed to load first half of declaration")
                        }
                    };

                    let var_type = match declaration_parts.next() {
                        Some(val) => val,
                        None => {
                            panic!("Failed to load variable value")
                        }
                    }.trim();

                    ast.lines.push(ASTLine {
                        verb: Verb::Set,
                        nouns: vec![var_name.to_string(), var_type.to_string(), val.to_string()],
                    })
                },
                '!' => {    // Mark statement (marker_name!)
                    ast.lines.push(ASTLine {
                        verb: Verb::Mark,
                        nouns: vec![simplified_line.trim().to_string()],
                    })
                }
                _ => {      // Return statement (var_name)
                    ast.lines.push(ASTLine {
                        verb: Verb::Return,
                        nouns: vec![line.trim().to_string(), (*type_of).to_string()],
                    })
                }
            }

            next_line = lines.next();
        }

        (function_name, Value::ASTRef(ast))
    }

    // Handle imports, load functions to AST, and handle loading constants to environment
    //**NOTE:** The AST must also break down if else statements and stuff into raw do blocks */
    fn load_function_AST(&mut self) -> () { // Self must be mutable because we mutate the Environment attribute
        let mut lines = (*self.file_contents).split("\n"); // iterator
        
        let mut next_line = lines.next();

        let mut vars = std::mem::take(&mut self.environment.vars); // Take control of vars so it doesnt conflict with call to self.fn_to_AST and borrow self

        while let Some(mut line) = next_line {  // Using while loop instead of traditional for_loop to add flexibility for looking ahead in the iterator           
            line = match line.split("//").next() {
                Some(val) => val,
                None => panic!("Failed to filter out comments"),
            }.trim();

            if line == "" {
                next_line = lines.next();
                continue;
            }

            let simplified_line = &line[0..line.len() - 1]; // Remove suffix

            let suffix = match line.chars().last() {
                Some(val) => val,
                None => panic!("Failed to read suffix")
            };

           let line = &line[..line.len()-1];

            match suffix {
                ';' => {    // Declarative line (x = 0)
                    let mut parts = simplified_line.split("=");

                    let var_declaration = match parts.next() {
                        Some(val) => val,
                        None => {
                            panic!("Failed to load first half of declaration")
                        }
                    };

                    let val = match parts.next() {
                        Some(val) => val,
                        None => {
                            panic!("Failed to load variable value")
                        }
                    }.trim();

                    let mut declaration_parts = var_declaration.split(":");

                    let var_name = match declaration_parts.next() {
                        Some(val) => val.trim(),
                        None => {
                            panic!("Failed to load first half of declaration")
                        }
                    };

                    let var_type = match declaration_parts.next() {
                        Some(val) => val,
                        None => {
                            panic!("Failed to load variable value")
                        }
                    }.trim();

                    vars.insert(var_name.to_string(), Value::parse(val, var_type));      
                    // PLAN: INSTEAD OF USING Value::parse REMOVE THE PARSE FUNCTION AND INSTEAD:
                    // Make the evaluate function static, and make this call the static foo
                    // And make a copy of the eval functoon in the ASTRunner which first checks for an existing var   
                },
                '{' => {    // Constructive line (if, function, etc.)
                    // Extract return type of function & parameters (syntax: function_name: return_type (param1: type, param2: type))
                    let mut parts = line.split("(");
                    let traits = match parts.next() {
                        Some(val) => val,
                        None => panic!("Failed to load function traits")
                    };
                    let mut params_string = match parts.next() {
                        Some(val) => val,
                        None => panic!("Failed to load function params")
                    }.to_string();
                    params_string = params_string.replace(")", "");
                    params_string = params_string.trim().to_string();

                    let mut split_params = params_string.split(",");
                    let mut params: HashMap<String, String> = HashMap::new();
                    while let Some(param) = split_params.next() {
                        let mut split_param = param.split(":");
                        let param_name = match split_param.next() {
                            Some(val) => val,
                            None => {
                                panic!("Failed to load param name");
                            }
                        };
                        let param_type = match split_param.next() {
                            Some(val) => val,
                            None => {
                                panic!("Failed to load param type");
                            }
                        };
                        params.insert(param_name.to_string(), param_type.to_string());
                    }

                    let mut trait_parts = traits.split(":");
                    let function_name = match trait_parts.next() {
                        Some(val) => val,
                        None => {
                            panic!("Failed to load function name")
                        }
                    };
                    let return_type = match trait_parts.next() {
                        Some(val) => val,
                        None => {
                            panic!("Failed to load function return type");
                        }
                    };

                    let mut function_contents = String::new();

                    // Load all lines until closing } to function_contents (**NOTE:** There is no need to worry about sub if, else, switch, statements, as they will use Do marker blocks)
                    while let Some(mut next_line) = lines.next() {
                        next_line = match next_line.split("//").next() {
                            Some(val) => val.trim(),
                            None => panic!("Failed to filter out comments"),
                        };

                        if next_line == "}" {
                            break;
                        }

                        function_contents += format!("{}\n", next_line.trim()).as_str();
                    }

                    let (function_name, AST_ref) = Self::fn_to_AST(function_contents, return_type.trim().to_string(), params, function_name.to_string());

                    vars.insert(function_name, AST_ref);
                },
                '!' => {    // Import

                },
                _ => {
                    panic!("Unknown suffix \"{:?}\", line \"{:?}\"", suffix, line)
                }
            };

            next_line = lines.next();   
        }

        self.environment.vars = vars; // Put vars back into self
    }
}

struct ASTRunner {
    ast: AST,
}

impl ASTRunner {
    fn new(ast: AST) -> Self {
        ASTRunner {
            ast: ast,
        }
    }

    fn evaluate(&self, value: &str, var_type: &str, env: Environment) -> Value {
        // Early return for "none"
        if value == "none" {
            return Value::Null;
        }

        let env_search = env.search_for_var(value.to_string());
        match env_search {
            Value::Undefined => {
                match var_type {
                    "int" => {
                        match value.parse::<i32>() {
                            Ok(val) => Value::Int(val),
                            Err(_) => env.search_for_var(value.to_string())
                        }
                    },
                    "float" => {
                        match value.parse::<f64>() {
                            Ok(val) => Value::Float(val),
                            Err(_) => env.search_for_var(value.to_string())
                        }
                    },
                    "bool" => {
                        if value == "true" {
                            return Value::Bool(true);
                        }
                        if value == "false" {
                            return Value::Bool(false);
                        }
                        let env_search = env.search_for_var(value.to_string());
                        Value::Bool(self.check_conditional(value, env))
                    },
                    _ => Value::Undefined
                }
            }
            _ => env_search,
        }
    }

    fn check_conditional(&self, conditional: &str, env: Environment) -> bool {
        let parts: Vec<&str> = conditional[3..].split(' ').collect();
        if parts == [""] {
            panic!("Conditional without body: '{}'", conditional);
        }

        let mut final_result = false;
        let mut check_type: Option<&str> = None;
        let mut check_obj: Option<Value> = None;
        let mut upper_check: Option<&str> = None;
        let upper_checks = ["and", "or"];
        let tags = ["not"];
        let mut tag: Option<&str> = None;
        let mut current = false;

        for part in parts {
            if part.contains('=') || part.contains('<') || part.contains('>') {
                check_type = Some(part);
            }
            else if upper_checks.contains(&part) {
                upper_check = Some(part);
            }
            else if tags.contains(&part) {
                tag = Some(part);
            }
            else if check_type.is_none() && !part.contains('=') && !part.contains('<') && !part.contains('>') {
                match part.split(':').collect::<Vec<&str>>().as_slice() {
                    [val, var_type] => {
                        check_obj = Some(self.evaluate(val, var_type, env.clone()));
                    }
                    _ => {
                        panic!("Typeless conditional object: '{}'", part);
                    }
                }
            }
            else {
                match check_type {
                    Some("==") => {
                        match part.split(':').collect::<Vec<&str>>().as_slice() {
                            [val, var_type] => {
                                let right_value = self.evaluate(val, var_type, env.clone());
                                current = check_obj.as_ref().unwrap() == &right_value;
                            }
                            _ => {
                                panic!("Typeless conditional check: '{}'", part);
                            }
                        }
                        check_type = None;
                        check_obj = None;
                    }
                    Some("<=") => {
                        match part.split(':').collect::<Vec<&str>>().as_slice() {
                            [val, var_type] => {
                                let right_value = self.evaluate(val, var_type, env.clone());
                                if let Some(ord) = check_obj.as_ref().unwrap().partial_cmp(&right_value) {
                                    current = ord != std::cmp::Ordering::Greater;
                                } else {
                                    current = false;
                                }
                            }
                            _ => {
                                panic!("Typeless conditional check: '{}'", part);
                            }
                        }
                        check_type = None;
                        check_obj = None;
                    }
                    Some(">=") => {
                        match part.split(':').collect::<Vec<&str>>().as_slice() {
                            [val, var_type] => {
                                let right_value = self.evaluate(val, var_type, env.clone());
                                if let Some(ord) = check_obj.as_ref().unwrap().partial_cmp(&right_value) {
                                    current = ord != std::cmp::Ordering::Less;
                                } else {
                                    current = false;
                                }
                            }
                            _ => {
                                panic!("Typeless conditional check: '{}'", part);
                            }
                        }
                        check_type = None;
                        check_obj = None;
                    }
                    Some("<") => {
                        match part.split(':').collect::<Vec<&str>>().as_slice() {
                            [val, var_type] => {
                                let right_value = self.evaluate(val, var_type, env.clone());
                                if let Some(ord) = check_obj.as_ref().unwrap().partial_cmp(&right_value) {
                                    current = ord == std::cmp::Ordering::Less;
                                } else {
                                    current = false;
                                }
                            }
                            _ => {
                                panic!("Typeless conditional check: '{}'", part);
                            }
                        }
                        check_type = None;
                        check_obj = None;
                    }
                    Some(">") => {
                        match part.split(':').collect::<Vec<&str>>().as_slice() {
                            [val, var_type] => {
                                let right_value = self.evaluate(val, var_type, env.clone());
                                if let Some(ord) = check_obj.as_ref().unwrap().partial_cmp(&right_value) {
                                    current = ord == std::cmp::Ordering::Greater;
                                } else {
                                    current = false;
                                }
                            }
                            _ => {
                                panic!("Typeless conditional check: '{}'", part);
                            }
                        }
                        check_type = None;
                        check_obj = None;
                    }
                    _ => {}
                }

                if tag == Some("not") {
                    current = !current;
                }
                tag = None;

                match upper_check {
                    None => {
                        final_result = current;
                    }
                    Some("and") => {
                        final_result = final_result && current;
                    }
                    Some("or") => {
                        final_result = final_result || current;
                    }
                    Some("xor") => {
                        final_result = final_result ^ current;
                    }
                    _ => {}
                }
            }
        }

        final_result
    }

    fn run(&mut self, parent_env: Environment, param_string: HashMap<String, String>) -> Value {
        let lines = &self.ast.lines;
        let mut env = Environment::new();
        
        // Load params to env
        let mut params: HashMap<String, Value> = HashMap::new();
        let param_types = &self.ast.params;
        for (param, string_val) in param_string {
            let type_of = match param_types.get(&param) {
                Some(val) => val,
                None => panic!("Non existent parameter \"{}\"", param)
            };

            let true_val = self.evaluate(string_val.as_str(), type_of, env.clone());
            params.insert(param, true_val);
        }

        env.parent = Some(Rc::new(parent_env));

        let mut index = 0;

        while let Some(line) = lines.get(index) {
            match line.verb {
                Verb::Set => {
                    let var_name = line.nouns.get(0).unwrap();
                    let var_type = line.nouns.get(1).unwrap();
                    let val = line.nouns.get(2).unwrap();

                    let evaluated_val = self.evaluate(val, var_type, env.clone());
                    env.vars.insert(var_name.clone(), evaluated_val);
                },
                Verb::Return => {
                    let val = line.nouns.get(0).unwrap();
                    let val_type = line.nouns.get(1).unwrap();

                    let evaluated_val = self.evaluate(val, val_type, env.clone());
                    return evaluated_val;
                },
                Verb::Mark => {
                    let marker_name = line.nouns.get(0).unwrap();
                    env.vars.insert(marker_name.clone(), Value::Int(index as i32));
                },
                Verb::Do => {
                    let marker_name = line.nouns.get(0).unwrap();
                    let conditional = line.nouns.get(1).unwrap();

                    if self.evaluate(conditional, "bool", env.clone()) == Value::Bool(true) || 
                       self.check_conditional(conditional, env.clone()) {
                        if marker_name.starts_with("~") {
                            let raw_marker_index = env.vars.get(&marker_name[1..]);

                            let true_marker_index = match raw_marker_index {
                                Some(Value::Int(val)) => *val,
                                _ => panic!("Provided marker points to a non integer index")
                            };

                            index = true_marker_index as usize;

                            continue;
                        }
                        
                        if marker_name.starts_with("*") {
                            let mut marker_index = 0;
                            for line in lines {
                                match line.verb {
                                    Verb::Mark => {
                                        let potential_marker_name = match line.nouns.get(0) {
                                            Some(val) => val,
                                            None => {
                                                panic!("Marker without name");
                                            }   
                                        };
            
                                        if potential_marker_name == marker_name {
                                            index += marker_index;
                                            break;
                                        }
                                    }
                                    _ => marker_index += 1,
                                }
                            }

                            continue;
                        }

                        let mut marker_index = 0;
                        for line in &lines[index..] {
                            match line.verb {
                                Verb::Mark => {
                                    let potential_marker_name = match line.nouns.get(0) {
                                        Some(val) => val,
                                        None => {
                                            panic!("Marker without name");
                                        }   
                                    };
        
                                    if potential_marker_name == marker_name {
                                        index += marker_index;
                                        break;
                                    }
                                }
                                _ => marker_index += 1,
                            }

                        }
                    }
                }
            }

            index += 1;
        }

        Value::Undefined
    } 
}

fn main() {
    let env = Environment::new();   // Generate global environment
    let mut generator = ASTGenerator::new("C:\\Users\\austi\\projects\\Luma\\plan.luma", env);
    generator.load_function_AST();   // Load AST using the environment stored in the generator
    let main_foo = match generator.environment.vars.get("main") {
        Some(val) => match val {
            Value::ASTRef(ast) => ast.clone(),
            _ => {
                panic!("Main not a function");
            }
        },
        None => {
            panic!("No main function");
        }
    };
    let mut runner = ASTRunner::new(main_foo);
    let result = runner.run(generator.environment, HashMap::new());
    println!("Value: {}", result.print_value());
    println!("Type: {}", result.print_type());
}