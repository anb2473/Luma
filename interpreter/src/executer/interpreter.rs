use crate::parser_core::ast::{AST, AST_type};
use crate::parser_core::value::{Value};
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

    pub fn run(&self) -> Value {
        for line in self.ast {
            match line.statement_type {
                AST_type::Set => {

                },
                _ => panic!("Unknown opperation"),
            };
        }
    }
}