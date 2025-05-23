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
            env: Environment::new(),
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
                            a = val.clone();
                        }
                        _ => {
                            match opp {
                                Verb::Add => {
                                    // Cast b to the type of a and perform addition
                                    let result = val.cast_to_type(&a);
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
                                    let result = val.cast_to_type(&a);
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
                                    let result = val.cast_to_type(&a);
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
                                    let result = val.cast_to_type(&a);
                                    a = match (a, result) {
                                        (Value::Int(a_val), Value::Int(b_val)) => Value::Int(a_val * b_val),
                                        (Value::Float(a_val), Value::Float(b_val)) => Value::Float(a_val * b_val),
                                        (Value::Str(a_val), Value::Str(b_val)) => Value::Undefined,
                                        (Value::Char(a_val), Value::Char(b_val)) => Value::Int(a_val as i32 * b_val as i32),
                                        _ => Value::Undefined,
                                    };
                                }
                                Verb::None => {
                                    a = val.clone();
                                }
                                Verb::Set => {
                                    // Handle setting variables
                                    if let Value::VarName(name) = &a {
                                        // TODO: Implement variable setting
                                        a = val.clone();
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

    pub fn run(&self) -> Value {
        for line in &self.ast.statements {
            match line.statement_type {
                AST_type::Set => {
                    let var_name = &line.a;
                    let expression = &line.b;

                    println!("{:?}", self.evaluate_expression(expression.to_vec()));
                },
                _ => panic!("Unknown opperation"),
            };
        }

        Value::Undefined
    }
}