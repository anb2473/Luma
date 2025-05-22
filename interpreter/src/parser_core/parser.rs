use crate::parser_core::lexer::Lexer;
use crate::parser_core::ast::{AST, AST_statement, AST_type, AST_object};
use crate::parser_core::tokenized::{Token, Verb};
use crate::parser_core::value;

pub struct Parser {
    lexer: Lexer,
}

impl Parser {
    pub fn new(lexer: Lexer) -> Self {
        Parser {
            lexer: lexer,
        }
    }

    pub fn run(&self) -> AST {
        let mut statements = Vec::new();

        for token_list in &self.lexer.tokenized_lines.lines {
            
        }

        AST { statements }
    }
}