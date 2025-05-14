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
    SET,
    RETURN,
}

struct ASTLine {
    pub verb: Verb,
    pub nouns: Vec<String>,
}

struct AST {
    pub lines: ASTLine,
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
    Environment(Rc<Environment>),   // Holds a smart pointer to an enviorenment (for classes)
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


// **NOTE:** When a request for a value in the enviorenment is made the system will start by searching through the raw enviorenment,
// Before following the pointers to each parent enviorenment and searching for the value there

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
        // Search through the parents stack starting at the local enviorenment until either no further parent is found or the value is found
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
}

impl ASTGenerator {
    fn new(path: &str) -> Self {
        let file_contents = match read_file(path) {
            Ok(val) => val,
            None => panic!("File not found")
        };

        ASTGenerator {
            file_contents: file_contents,
        }
    }

    
    fn fn_to_AST(&self, function: String) -> () {
        
    }

    // Handle imports, load functions to AST, and handle loading constants to enviorenment
    //**NOTE:** The AST must also break down if else statements and stuff into raw do blocks */
    fn load_function_AST(&self, env: &Enviorenment) -> () {
        let lines = file_contents.split("\n"); // iterator
        
        let next_line = lines.next();

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

                },
                '{' => {    // Constructive line (if, function, etc.)
                    
                },
                '!' => {    // Import

                },
                _ => {      // Return

                }
            };
        }
    }
}

fn main() {
    let env = Environment::new();   // Generate global environment
    let generator = ASTGenerator::new();
    generator.load_function_AST(env);   // Load AST to previosly defined env
}