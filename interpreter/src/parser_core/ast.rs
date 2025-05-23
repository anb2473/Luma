use crate::parser_core::value;
use crate::parser_core::tokenized;

#[derive(Clone, Debug)]
pub enum AST_type {
    Add,
    Sub,
    Mult,
    Div,
    Set,
}

#[derive(Clone, Debug)]
pub struct AST_statement {
    pub statement_type: AST_type,
    pub a: value::Value,
    pub b: Vec<tokenized::Token>,
}

pub struct AST {
    pub statements: Vec<AST_statement>,
}