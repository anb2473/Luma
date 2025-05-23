use crate::parser_core::lexer::Lexer;
use crate::parser_core::ast::{AST, AST_statement, AST_type};
use crate::parser_core::tokenized::{Token, Verb};
use crate::parser_core::value;
use crate::parser_core::tokenized;
use crate::parser_core;

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
        let mut statements: Vec<AST_statement> = Vec::new();

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
                    // Extraction format for right side: [Noun, Verb, [Noun, Verb]]
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
                    
                    statements.push(AST_statement {
                        statement_type: statement_type,
                        a: a,
                        b: right_hand,
                    });
                },
                tokenized::Suffix::Return => {
                    statements.push(AST_statement {
                        statement_type: AST_type::Return,
                        a: value::Value::Undefined,
                        b: token_list.objects.clone(),
                    });
                }
                _ => {
                    panic!("Unknown suffix");
                }
            };
        }

        AST { statements }
    }
}