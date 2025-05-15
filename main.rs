use std::collections::HashMap;
use std::rc::Rc;

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

enum Verb {
    Set,     // x = 0;
    Return,  // x
    Mark,  // my_import!
    Do,
}

struct ASTLine {
    pub verb: Verb,
    pub nouns: Vec<String>,
}

struct AST {
    pub lines: Vec<ASTLine>,
    pub params: HashMap<String, String>,
}

// Value enum will hold all the referencable types

#[derive(Debug, Clone)]
enum Value {
    Int(i64),
    Float(f64),
    Str(String),
    Bool(bool),
    List(Vec<Value>),
    FunctionRef(Box<dyn Fn(Vec<Value>) -> Value>),   // Holds a function reference (This will allow for you to call Rust functions)
    ASTRef(AST), // Holds a function reference (This will allow for you to call Luma functions)
    Environment(Rc<Environment>),   // Holds a smart pointer to an environment (for classes)
    Null,
}

// For converting raw string nouns to actual values

trait ToValue {
    fn to_value(&self, val: String) -> Value;
}

impl Value {
    // Function implementation to call casting function between types

    pub fn cast_to<T: ToValue>(self) -> Value {
        T::to_value(self)
    }

    // Functions to generate a value from a str representation

    // Orginization function

    pub fn parse(input: &str, type_of: &str) {
        match type_of {
            "int" => parse_int(input),
            "str" => parse_str(input),
            "float" => parse_float(input),
            "bool" => parse_bool(input),
        }
    }

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
}


// **NOTE:** When a request for a value in the environment is made the system will start by searching through the raw environment,
// Before following the pointers to each parent environment and searching for the value there

#[derive(Clone, Debug)] // Derive Clone if you want to clone instances of Environment
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

    fn search_for_var(&self, name) -> Value {
        // Search through the parents stack starting at the local environment until either no further parent is found or the value is found
        let mut parent = self;
        while let Some(next_parent) = parent {
            if let Some(val) = next_parent.vars.get(name) {
                val
            }

            parent = next_parent.parent;
        }

        Value::Null
    }
}

struct ASTGenerator {
    file_contents: String,
    environment: environment,
}

impl ASTGenerator {
    fn new(path: &str, environment: environment) -> Self {
        let file_contents = match read_file(path) {
            Ok(val) => val,
            None => panic!("File not found")
        };

        ASTGenerator {
            file_contents: file_contents,
            environment: environment,
        }
    }

    
    fn fn_to_AST(&self, function: String, type_of: String, params: HashMap<String, String>, function_name: String) -> () {
        let lines = function.split("\n");

        let mut next_line = lines.next();

        let ast = AST { lines: Vec::new(), params }

        while let Some(line) = next_line {
            simplified_line = &line[0..line.len() - 1]; // Remove suffix

            let suffix = match line.chars().last() {
                Some(val) => val,
                None => panic!("Failed to read suffix")
            };
            
            match suffix {
                '?' => {    // Do statement (syntax: marker_name conditional?)
                    split_line = simplified_line.splitn(1, " ")

                    ast.lines.push(ASTLine {
                        verb: Verb::Do,
                        nouns: vec![match split_line.next() {
                            Some(val) => val,
                            None => panic!("Failed to load marker name of do statement")
                        }, match split_line.next() {
                            Some(val) => val,
                            None => panic!("Failed to load conditional statement of do statement")
                        }],
                    })
                },
                ';' => {    // Set statement (syntax: var: type = val;)
                     let parts = line.split("=")

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
                    }.trim()

                    let declaration_parts = var_declaration.split(":")

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
                    }.trim()

                    ast.lines.push(ASTLine {
                        verb: Verb::Set,
                        nouns: vec![var_name.to_string(), var_type.to_string(), val.to_string()],
                    })
                },
                '!' => {    // Mark statement (marker_name!)
                    ast.lines.push(ASTLine {
                        verb: Verb::Mark,
                        nouns: vec![simplified_line.trim()],
                    })
                }
                _ => {      // Return statement (var_name)
                    ast.lines.push(ASTLine {
                        verb: Verb::Return,
                        nouns: vec![simplified_line.trim(), type_of],
                    })
                }
            }
        }

        self.environment.vars.insert(function_name, ast)
    }

    // Handle imports, load functions to AST, and handle loading constants to environment
    //**NOTE:** The AST must also break down if else statements and stuff into raw do blocks */
    fn load_function_AST(&self, env: Environment) -> () {
        let lines = file_contents.split("\n"); // iterator
        
        let mut next_line = lines.next();

        while let Some(line) = next_line {  // Using while loop instead of traditional for_loop to add flexibility for looking ahead in the iterator
            if line == "" {
                continue;
            }

            let suffix = match line.chars().last() {
                Ok(val) => val,
                None => panic!("Failed to read suffix")
            };

            match suffix {
                ';' => {    // Declarative line (x = 0)
                    let parts = line.split("=")

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
                    }.trim()

                    let declaration_parts = var_declaration.split(":")

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
                    }.trim()

                    self.environment.vars.insert(var_name, Value::parse(val, var_type));
                },
                '{' => {    // Constructive line (if, function, etc.)
                    // Extract return type of function & parameters (syntax: function_name: return_type (param1: type, param2: type))
                    let parts = next_line.split("(");
                    let traits = match parts.next() {
                        Some(val) => val,
                        None => panic!("Failed to load function traits")
                    }
                    let params_string = match parts.next() {
                        Some(val) => val,
                        None => panic!("Failed to load function params")
                    }

                    let split_params = parts.split(",");
                    let params = HashMap<String, String>::new()
                    while let some(param) = split_params.next() {
                        let split_param = param.split(",");
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
                        params.insert(param_name, param_type);
                    }

                    let trait_parts = traits.split(":");
                    let function_name = match trait_parts.next() {
                        Some(val) => val,
                        None => {
                            panic!("Failed to load function name")
                        }
                    };
                    let return_type = match trait.parts.next() {
                        Some(val) => val,
                        None => {
                            panic!("Failed to load function return type");
                        }
                    };

                    let function_contents = String::new();

                    // Load all lines until closing } to function_contents (**NOTE:** There is no need to worry about sub if, else, switch, statements, as they will use Do marker blocks)
                    while let Some(next_line) = lines.next() {
                        if next_line == "}" {
                            break;
                        }

                        function_contents += next_line.trim();
                    }

                    self.fn_to_AST(function_contents, return_type, params, function_name);
                },
                '!' => {    // Import

                },
                _ => {
                    panic!("Unknown suffix")
                }
            };

            next_line = lines.next();   
        }
    }
}

fn main() {
    let env = Environment::new();   // Generate global environment
    let generator = ASTGenerator::new("C:\\Users\\austi\\projects\\Luma\\plan.luma", env);
    generator.load_function_AST(env);   // Load AST to previosly defined env
}