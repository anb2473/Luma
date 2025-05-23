use crate::parser_core::lexer::Lexer;
use crate::parser_core::ast::{AST, AST_statement, AST_type, AST_object};
use crate::parser_core::tokenized::{Token, Verb};
use crate::parser_core::value;
use crate::parser_core::tokenized;

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
            let suffix = match &token_list.suffix {
                Some(val) => val,
                None => panic!("Failed to load line suffix"),
            };

            match suffix {
                tokenized::Suffix::Set => 
                    let statement_type = AST_type::Set;
                    let eq_side = false;
                    // **GOAL** We need to identify the variable name, and from there evaluate expression
                    for token in token_list.objects {
                        match token {
                            tokenized::Token::Noun(val) => {
                                match val {
                                    value::Value::VarName(name) => {
                                        let a = value::Value::VarName(name);
                                    }
                                    _ => {
                                        panic!("Unknown noun")
                                    }
                                }
                            },
                            tokenized::Token::Verb(val) => {
                                match val {
                                    tokenized::Verb::Set => {
                                        eq_side = true;
                                    }
                                }
                            }
                        }
                    }
                },
                _ => {
                    panic!("Unknown suffix")
                }
            };
        }

        AST { statements }
    }
}