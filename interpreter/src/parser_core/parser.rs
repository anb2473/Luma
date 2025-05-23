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
                tokenized::Suffix::Set => {
                    let statement_type = AST_type::Set;
                    let mut eq_side = false;
                    // **GOAL** We need to identify the variable name, and from there evaluate expression
                    // Load variable name and right hand expression and extract parenthesees
                    let mut right_hand: Vec<Token> = Vec::new();
                    let mut a = value::Value::VarName(String::new());
                    for token in &token_list.objects {
                        match token {
                            tokenized::Token::Noun(val) => {
                                match val {
                                    value::Value::VarName(name) => {
                                        if !eq_side {
                                            a = val.clone();
                                        }
                                        else {
                                            right_hand.push(Token::Noun(val.clone()));
                                        }
                                    }
                                    _ => {
                                        right_hand.push(Token::Noun(val.clone()));
                                    }
                                }
                            },
                            tokenized::Token::Verb(val) => {
                                match val {
                                    tokenized::Verb::Set => {
                                        eq_side = true;
                                    },
                                    _ => {
                                        right_hand.push(Token::Verb(val.clone()));
                                    }
                                }
                            }
                        }
                    }

                    // Take right side of the expression and build a AST statement from it
                },
                _ => {
                    panic!("Unknown suffix")
                }
            };
        }

        AST { statements }
    }
}