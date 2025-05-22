use crate::parser_core::value;

#[derive(Debug)]
pub enum AST_type {
    Add,
    Sub,
    Mult,
    Div,
}

#[derive(Debug)]
pub enum AST_object {
    Statement(Box<AST_statement>),
    Object(value::Value),
}

#[derive(Debug)]
pub struct AST_statement {
    pub statement_type: AST_type,
    pub a: AST_object,
    pub b: Option<AST_object>,
}

pub struct AST {
    pub statements: Vec<AST_statement>,
}