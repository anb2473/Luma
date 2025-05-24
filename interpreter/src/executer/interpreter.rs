use crate::parser_core::ast::{AST, AST_type};
use crate::parser_core::value::{Value};
use crate::parser_core::tokenized::{Token, Verb};
use crate::executer::runtime::environment::{Environment};

pub struct Interpreter {
    ast: AST,
    env: Environment
}

impl Interpreter {
    pub fn new(ast: AST) -> Self {
        Interpreter {
            ast: ast,
            env: Environment::new(None),
        }
    }

    fn evaluate_expression(&self, expression: Vec<Token>) -> Value {
        let mut iter = expression.iter();

        let mut next_token = iter.next();

        let mut a = Value::Undefined;
        let mut opp = Verb::None;

        while let Some(token) = next_token {
            match token {
                Token::Noun(val) => {
                    match a {
                        Value::Undefined => {
                            a = match val.clone() {
                                Value::VarName(name) => self.env.search_for_var(name.to_string()),
                                _ => val.clone(),
                            };
                        }
                        _ => {
                            let resolved_val = match val {
                                Value::VarName(name) => self.env.search_for_var(name.clone()),
                                _ => val.clone(),
                            };
                            
                            match opp {
                                Verb::Add => {
                                    // Cast b to the type of a and perform addition
                                    let result = resolved_val.cast_to_type(&a);
                                    a = match (a, result) {
                                        (Value::Int(a_val), Value::Int(b_val)) => Value::Int(a_val + b_val),
                                        (Value::Float(a_val), Value::Float(b_val)) => Value::Float(a_val + b_val),
                                        (Value::Str(a_val), Value::Str(b_val)) => Value::Str(a_val + &b_val),
                                        (Value::Char(a_val), Value::Char(b_val)) => Value::Int(a_val as i32 + b_val as i32),
                                        _ => Value::Undefined,
                                    };
                                }
                                Verb::Sub => {
                                    // Cast b to the type of a and perform subtraction
                                    let result = resolved_val.cast_to_type(&a);
                                    a = match (a, result) {
                                        (Value::Int(a_val), Value::Int(b_val)) => Value::Int(a_val - b_val),
                                        (Value::Float(a_val), Value::Float(b_val)) => Value::Float(a_val - b_val),
                                        (Value::Str(a_val), Value::Str(b_val)) => Value::Undefined,
                                        (Value::Char(a_val), Value::Char(b_val)) => Value::Int(a_val as i32 - b_val as i32),
                                        _ => Value::Undefined,
                                    };
                                }
                                Verb::Div => {
                                    // Cast b to the type of a and perform division
                                    let result = resolved_val.cast_to_type(&a);
                                    a = match (a, result) {
                                        (Value::Int(a_val), Value::Int(b_val)) => Value::Int(a_val / b_val),
                                        (Value::Float(a_val), Value::Float(b_val)) => Value::Float(a_val / b_val),
                                        (Value::Str(a_val), Value::Str(b_val)) => Value::Undefined,
                                        (Value::Char(a_val), Value::Char(b_val)) => Value::Int(a_val as i32 / b_val as i32),
                                        _ => Value::Undefined,
                                    };
                                }
                                Verb::Mult => {
                                    // Cast b to the type of a and perform multiplication
                                    let result = resolved_val.cast_to_type(&a);
                                    a = match (a, result) {
                                        (Value::Int(a_val), Value::Int(b_val)) => Value::Int(a_val * b_val),
                                        (Value::Float(a_val), Value::Float(b_val)) => Value::Float(a_val * b_val),
                                        (Value::Str(a_val), Value::Str(b_val)) => Value::Undefined,
                                        (Value::Char(a_val), Value::Char(b_val)) => Value::Int(a_val as i32 * b_val as i32),
                                        _ => Value::Undefined,
                                    };
                                }
                                Verb::None => {
                                    a = resolved_val.clone();
                                }
                                Verb::Set => {
                                    // Handle setting variables
                                    if let Value::VarName(name) = &a {
                                        // TODO: Implement variable setting
                                        a = resolved_val.clone();
                                    } else {
                                        a = Value::Undefined;
                                    }
                                }
                            }
                        }
                    }
                }
                Token::Verb(val) => {
                    opp = val.clone();
                }
            }

            next_token = iter.next();
        }

        a
    }

    pub fn run(&mut self) -> Value {
        for line in &self.ast.statements {
            match line.statement_type {
                AST_type::Set => {
                    let var_name = match &line.a {
                        Value::VarName(name) => name,
                        _ => panic!("Non variable name")
                    };
                    let expression = &line.b;

                    self.env.vars.insert(var_name.clone(), self.evaluate_expression(expression.clone()));
                },
                AST_type::Return => {
                    let val = &line.b; 
                    return self.evaluate_expression(val.clone());
                }
                _ => panic!("Unknown opperation"),
            };
        }

        Value::Undefined
    }
}