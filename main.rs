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

#[derive(Clone)]
enum Verb {
    Set,     // x = 0;
    Return,  // x
    Mark,  // my_import!
    Do,
}

#[derive(Clone)]
struct ASTLine {
    pub verb: Verb,
    pub nouns: Vec<String>,
}

#[derive(Clone)]
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
    Null,
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
        Value::Null
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

    // Self must be mutable as we mutate the enviorenment attribute
    fn fn_to_AST(function: String, type_of: String, params: HashMap<String, String>, function_name: String) -> (String, Value) {
        let mut lines = function.split("\n");

        let mut next_line = lines.next();

        let mut ast = AST { lines: Vec::new(), params };

        while let Some(line) = next_line {
            let simplified_line = &line[0..line.len() - 1]; // Remove suffix

            let suffix = match line.chars().last() {
                Some(val) => val,
                None => panic!("Failed to read suffix")
            };
            
            match suffix {
                '?' => {    // Do statement (syntax: marker_name conditional?)
                    let mut split_line = simplified_line.splitn(1, " ");

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
                     let mut parts = line.split("=");

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

                                        let var_declaration = match parts.next() {
                        Some(val) => val,
                        None => {
                            panic!("Failed to load first half of declaration")
                        }
                    };

                    let var_name = match declaration_parts.next() {
                        Some(val) => val,
                        None => {
                            panic!("Failed to load first half of declaration")
                        }
                    }.trim();

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
                        nouns: vec![simplified_line.trim().to_string(), (*type_of).to_string()],
                    })
                }
            }
        }

        (function_name, Value::ASTRef(ast))
    }

    // Handle imports, load functions to AST, and handle loading constants to environment
    //**NOTE:** The AST must also break down if else statements and stuff into raw do blocks */
    fn load_function_AST(&mut self) -> () { // Self must be mutable because we mutate the enviorenment attribute
        let mut lines = (*self.file_contents).split("\n"); // iterator
        
        let mut next_line = lines.next();

        let mut vars = std::mem::take(&mut self.environment.vars); // Take control of vars so it doesnt conflict with call to self.fn_to_AST and borrow self

        while let Some(line) = next_line {  // Using while loop instead of traditional for_loop to add flexibility for looking ahead in the iterator
            if line == "" {
                continue;
            }

            let suffix = match line.chars().last() {
                Some(val) => val,
                None => panic!("Failed to read suffix")
            };

            match suffix {
                ';' => {    // Declarative line (x = 0)
                    let mut parts = line.split("=");

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

                                        let var_declaration = match parts.next() {
                        Some(val) => val,
                        None => {
                            panic!("Failed to load first half of declaration")
                        }
                    };

                    let var_name = match declaration_parts.next() {
                        Some(val) => val,
                        None => {
                            panic!("Failed to load first half of declaration")
                        }
                    }.trim();

                    let var_type = match declaration_parts.next() {
                        Some(val) => val,
                        None => {
                            panic!("Failed to load variable value")
                        }
                    }.trim();

                    vars.insert(var_name.to_string(), Value::parse(val, var_type));         
                },
                '{' => {    // Constructive line (if, function, etc.)
                    // Extract return type of function & parameters (syntax: function_name: return_type (param1: type, param2: type))
                    let mut parts = line.split("(");
                    let traits = match parts.next() {
                        Some(val) => val,
                        None => panic!("Failed to load function traits")
                    };
                    let params_string = match parts.next() {
                        Some(val) => val,
                        None => panic!("Failed to load function params")
                    }.replace(")", "");

                    let mut split_params = params_string.split(",");
                    let mut params: HashMap<String, String> = HashMap::new();
                    while let Some(param) = split_params.next() {
                        let mut split_param = param.split(",");
                        let param_name = match split_param.next() {
                            Some(val) => val,
                            None => {
                                panic!("Failed to load param name");
                            }
                        };
                        let param_type = match split_param.next() {
                            Some(val) => val,
                            None => {
                                panic!("Failed to load param name");
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
                    while let Some(next_line) = lines.next() {
                        if next_line == "}" {
                            break;
                        }

                        function_contents += next_line.trim();
                    }

                    let (function_name, AST_ref) = Self::fn_to_AST(function_contents, return_type.to_string(), params, function_name.to_string());

                    vars.insert(function_name, AST_ref);
                },
                '!' => {    // Import

                },
                _ => {
                    panic!("Unknown suffix")
                }
            };

            next_line = lines.next();   
        }

        self.environment.vars = vars; // Put vars back into self
    }
}

fn main() {
    let env = Environment::new();   // Generate global environment
    let mut generator = ASTGenerator::new("C:\\Users\\austi\\projects\\Luma\\plan.luma", env);
    generator.load_function_AST();   // Load AST using the environment stored in the generator
}