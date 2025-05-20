use std::collections::{HashMap, HashSet};
use std::rc::Rc;
use std::path::{Path, PathBuf};
use std::fs::{self, File}; 
use std::io::{self, Read, Write};
use std::fmt::Debug;
use serde_json::{Result as JsonResult}; 
use serde_json::Value as JsonValue;
use serde::{Serialize};

fn get_file_name<P: AsRef<Path>>(path: P) -> Option<String> {
    path.as_ref()
        .file_stem()             // Gets the file name without extension
        .and_then(|s| s.to_str()) // Convert OsStr to &str
        .map(|s| s.to_string())   // Convert to String
}

/// Reads a JSON file at the given path and returns a HashMap
fn read_json<P: AsRef<Path>>(path: P) -> Result<HashMap<String, JsonValue>, io::Error> {
    let file_content = fs::read_to_string(path)?;
    let map: HashMap<String, JsonValue> = serde_json::from_str(&file_content)?;
    Ok(map)
}

fn list_files<P: AsRef<Path>>(dir: P) -> io::Result<Vec<PathBuf>> {
    let mut files = Vec::new();

    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_file() {
            files.push(path);
        }
    }

    Ok(files)
}

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

fn execute_rust_program(file_path: &str, args: Vec<Value>, env: Option<Environment>) -> Result<Value, io::Error> {
    // Serialize all arguments
    let serialized_args: Vec<String> = args.iter().map(|v| v.serialize()).collect();
    
    // Get the executable path by removing .rs extension
    let executable_path = file_path.trim_end_matches(".rs");
    
    // First compile the Rust file
    let compile_output = std::process::Command::new("rustc")
        .arg(file_path)
        .output()?;

    if !compile_output.status.success() {
        let stderr = String::from_utf8_lossy(&compile_output.stderr).to_string();
        return Err(io::Error::new(io::ErrorKind::Other, format!("Compilation failed: {}", stderr)));
    }

    // Then execute the compiled program with the serialized arguments
    let output = std::process::Command::new(executable_path)
        .args(&serialized_args)
        .output()?;

    // Convert output to string
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();

    if output.status.success() {
        let mut split_stdout = stdout.split(":");
        let stdout_val = match split_stdout.next() {
            Some(val) => val,
            None => panic!("Failed to load stdout value")
        };
        let stdout_type = match split_stdout.next() {
            Some(val) => val,
            None => panic!("Failed to load stdout type")
        };
        Ok(evaluate(stdout_val, stdout_type, env))
    } else {
        Err(io::Error::new(io::ErrorKind::Other, stderr))
    }
}

// Abstract syntax tree (AST): on compilation all the lines in the functions will be compiled to the AST format before execution
// AST is a Vec<Vec<String>, Which contains a Vec of lines, with a sub Vec of the parts of the line
// Each line will be split into a verb and nouns

#[derive(Serialize, Clone, Debug)]
enum Verb {
    Set,     // x = 0;
    Return,  // x
    Mark,    // my_import!
    Do,
}

#[derive(Serialize, Clone, Debug)]
struct ASTLine {
    pub verb: Verb,
    pub nouns: Vec<String>,
}

#[derive(Serialize, Clone, Debug)]
struct AST {
    pub lines: Vec<ASTLine>,
    pub params: Vec<String>,
}

// Value enum will hold all the referencable types

#[derive(Serialize, Clone)]
enum Value {
    Int(i32),
    Float(f64),
    Str(String),
    Bool(bool),
    List(Vec<Value>),
    ASTRef(AST), // Holds a function reference (This will allow for you to call Luma functions), **NOTE:** Rust functions are called as entire files
    #[serde(skip)]
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
    fn to_value(value: Value) -> Value;
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
            Value::ASTRef(_) => String::from("ast"),
            Value::Environment(_) => String::from("environment"),
            Value::Undefined => String::from("undefined"),
            Value::Null => String::from("null"),
        }
    }

    pub fn serialize(&self) -> String {
        match self {
            Value::Int(i) => i.to_string(),
            Value::Float(f) => f.to_string(),
            Value::Str(s) => format!("\"{}\"", s),
            Value::Bool(b) => b.to_string(),
            Value::List(l) => {
                let items: Vec<String> = l.iter().map(|v| v.serialize()).collect();
                format!("[{}]", items.join(","))
            },
            Value::ASTRef(ast) => serde_json::to_string(ast).unwrap_or_else(|_| "null".to_string()),
            Value::Environment(_) => "null".to_string(), // Skip environment serialization
            Value::Undefined => "undefined".to_string(),
            Value::Null => "null".to_string(),
        }
    }
}

fn evaluate(value: &str, var_type: &str, env: Option<Environment>) -> Value {
    if let Some(ref env) = env {
        if value.ends_with(")") {
            let mut split_value = value.split("(");
            let mut ref_name = match split_value.next() {
                Some(val) => val,
                None => {
                    panic!("Failed to load function reference name");
                },
            };

            let mut split_fn_name = ref_name.trim().split("::");

            let pk_name = match split_fn_name.next() {
                Some(val) => val,
                None => panic!("Failed to load package name")
            };

            let fn_name = match split_fn_name.next() {
                Some(val) => val,
                None => panic!("Failed to load package name")
            };

            let pk_fns = match env.dependencies.get(pk_name) {
                Some(val) => val,
                None => panic!("Failed to load package functions")
            };

            let fn_path = match pk_fns.functionalities.get(fn_name) {
                Some(val) => val,
                None => panic!("Failed to load function path")
            };

            let params = match split_value.next() {
                Some(val) => val,
                None => {
                    panic!("Failed to load parameters");
                },
            };

            let mut split_params = params.split(",");

            let mut param_vec: Vec<Value> = Vec::new();

            for param in split_params {
                let mut split_param = param.split(":");
                
                let param_value = match split_param.next() {
                    Some(val) => val,
                    None => panic!("Failed to load parameter value")
                };

                let param_type = match split_param.next() {
                    Some(val) => val,
                    None => panic!("Failed to load parameter type")
                };

                let true_param = evaluate(param_value, param_type, Option::from(env.clone()));

                param_vec.push(true_param);
            }

            if value.starts_with("#") { // Function is imported (Can be either Rust or Luma)
                // Execute the Rust program with the parameters
                return match execute_rust_program(fn_path, param_vec, Option::from(env.clone())) {
                    Ok(output) => output,
                    Err(err) => {
                        panic!("{}", err);
                    },
                };
            }
            else {  // Function is defined in the file (MUST be a Luma function)
                let fn_AST = match env.search_for_var(fn_name.to_string()) {
                    Value::ASTRef(val) => val,
                    _ => {
                        panic!("Attempting to execute non function value as a function");
                    },
                };

                let fn_params = fn_AST.params.clone();

                // Create a HashMap mapping parameter names to their values
                let mut param_map: HashMap<String, Value> = HashMap::new();
                for (param_name, param_value) in fn_params.iter().zip(param_vec.iter()) {
                    param_map.insert(param_name.clone(), param_value.clone());
                }

                let mut runner = ASTRunner::new(fn_AST);
                let result = runner.run(env.clone(), param_map);
                return result;
            }
        } else {
            // First check if we have an environment and if the value exists in it
            let env_search = env.search_for_var(value.to_string());
            if env_search != Value::Undefined {
                return env_search;
            }
        }
    }

    // If no environment or value not found, proceed with normal evaluation
    match var_type {
        "int" => {
            match value.parse::<i32>() {
                Ok(val) => Value::Int(val),
                Err(err) => panic!("{}", err)
            }
        },
        "float" => {
            match value.parse::<f64>() {
                Ok(val) => Value::Float(val),
                Err(err) => panic!("{}", err)
            }
        },
        "bool" => {
            if value.starts_with("if") {
                // For conditional expressions, we need the environment
                if let Some(env) = env {
                    // We'll need to implement check_conditional as a standalone function
                    Value::Bool(check_conditional(value, env))
                } else {
                    panic!("Conditional expression requires environment");
                }
            } else if value == "true" {
                Value::Bool(true)
            } else if value == "false" {
                Value::Bool(false)
            } else {
                Value::Undefined
            }
        },
        "str" => {
            Value::Str(value.trim().to_string())
        },
        "list" => {
            let vec_value = value
                .split(',')
                .filter_map(|s| {
                    let mut parts = s.split(':').map(str::trim);
                    let key = parts.next()?;
                    let val = parts.next()?;
                    Some(evaluate(val, key, env.clone()))
                })
                .collect();

            Value::List(vec_value)
        }
        "undefined" => {
            Value::Undefined
        }
        "env" => {
            Value::Environment(Rc::new(Environment::new(None)))
        }
        "none" => {
            Value::Null
        },
        _ => Value::Undefined
    }
}

// Move check_conditional to be a standalone function
fn check_conditional(conditional: &str, env: Environment) -> bool {
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
                    check_obj = Some(evaluate(val, var_type, Some(env.clone())));
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
                            let right_value = evaluate(val, var_type, Some(env.clone()));
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
                            let right_value = evaluate(val, var_type, Some(env.clone()));
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
                            let right_value = evaluate(val, var_type, Some(env.clone()));
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
                            let right_value = evaluate(val, var_type, Some(env.clone()));
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
                            let right_value = evaluate(val, var_type, Some(env.clone()));
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

// **NOTE:** When a request for a value in the environment is made the system will start by searching through the raw environment,
// Before following the pointers to each parent environment and searching for the value there

#[derive(Clone)] // Derive Clone if you want to clone instances of Environment
struct Environment {
    vars: HashMap<String, Value>,    // Var name to Value (from enum) to allow fast look up times
    parent: Option<Rc<Environment>>, // Smart pointer back to parent environment to allow nested environments
    dependencies: HashMap<String, String>,     // Stores all of the programs dependencies as hash maps of file names (function names) to file paths
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

        let mut ast = AST { lines: Vec::new(), params: Vec::new() };

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

        //    let pk_manager = match read_json("pk_manager.json") {
        //         Ok(val) => val,
        //         Err(err) => {
        //             panic!("{}", err);
        //         },
        //    };

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

                    vars.insert(var_name.to_string(), evaluate(val, var_type, None));  
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
                    let mut params: Vec<String> = Vec::new();
                    while let Some(param) = split_params.next() {
                        let param_name = param.trim();
                        params.push(param_name.to_string());
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

                    let (function_name, AST_ref) = Self::fn_to_AST(function_contents, return_type.trim().to_string(), HashMap::new(), function_name.to_string());

                    vars.insert(function_name, AST_ref);
                },
                '!' => {    // Import
                    // let pk_name = simplified_line.trim();
                    // let pk_path = match pk_manager.get(pk_name) {
                    //     Some(val) => val,
                    //     None => {
                    //         panic!("Package import not found \"{}\"", pk_name);
                    //     },
                    // };

                    // let pk_ignore_contents = match read_file(format!("{}\\.pkignore", pk_path).as_str()) {
                    //     Ok(val) => val,
                    //     Err(err) => {
                    //         panic!("{}", err);
                    //     },
                    // };

                    // let pk_ignore: HashSet<&str> = pk_ignore_contents.lines().map(str::trim).collect(); // HashSet of &str (O(1) efficiency)
                    
                    // let sub_files = match list_files(pk_path.to_string()) {
                    //     Ok(val) => val,
                    //     Err(err) => {
                    //         panic!("{}", err);
                    //     }
                    // };

                    // let mut functionalities: HashMap<String, String> = HashMap::new();
                    // for file_buf in sub_files {
                    //     if let Some(file) = file_buf.to_str() {
                    //         if !pk_ignore.contains(file) {
                    //             let file_name = match get_file_name(file) {
                    //                 Some(val) => val,
                    //                 None => {
                    //                     panic!("Failed to extract file name")
                    //                 },
                    //             };
                    //             functionalities.insert(file_name, file.to_string());
                    //         }
                    //     }
                    // }
                    // self.environment.dependencies.insert(pk_name.to_string(), Functionalities {
                    //     functionalities: functionalities,
                    // });

                    let file_path = simplified_line.trim();

                    let base_name = get_file_name(file_path);

                    self.environment.dependencies.insert(nase_name, file_path);
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

    fn run(&mut self, parent_env: Environment, params: HashMap<String, Value>) -> Value {
        let lines = &self.ast.lines;
        let mut env = Environment::new(None);
        
        // Load params to env
        // let mut params: HashMap<String, Value> = HashMap::new();
        // let param_types = &self.ast.params;
        // for (param, string_val) in param_string {
        //     let type_of = match param_types.get(&param) {
        //         Some(val) => val,
        //         None => panic!("Non existent parameter \"{}\"", param)
        //     };

        //     let true_val = evaluate(string_val.as_str(), type_of, Some(env.clone()));
        //     params.insert(param, true_val);
        // }

        env.parent = Some(Rc::new(parent_env));
        env.vars.extend(params);

        let mut index = 0;

        while let Some(line) = lines.get(index) {
            match line.verb {
                Verb::Set => {
                    let var_name = line.nouns.get(0).unwrap();
                    let var_type = line.nouns.get(1).unwrap();
                    let val = line.nouns.get(2).unwrap();

                    let evaluated_val = evaluate(val, var_type, Some(env.clone()));
                    env.vars.insert(var_name.clone(), evaluated_val);
                },
                Verb::Return => {
                    let val = line.nouns.get(0).unwrap();
                    let val_type = line.nouns.get(1).unwrap();

                    let evaluated_val = evaluate(val, val_type, Some(env.clone()));
                    return evaluated_val;
                },
                Verb::Mark => {
                    let marker_name = line.nouns.get(0).unwrap();
                    env.vars.insert(marker_name.clone(), Value::Int(index as i32));
                },
                Verb::Do => {
                    let marker_name = line.nouns.get(0).unwrap();
                    let conditional = line.nouns.get(1).unwrap();

                    if evaluate(conditional, "bool", Some(env.clone())) == Value::Bool(true) || 
                       check_conditional(conditional, env.clone()) {
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
    let env = Environment::new(None);   // Generate global environment
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