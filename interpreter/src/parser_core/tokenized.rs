use crate::parser_core::value;

#[derive(Clone, Debug)]
pub enum Verb {
    Add,
    Sub,
    Mult,
    Div,
}

#[derive(Debug)]
pub enum Token {
    Verb(Verb),
    Noun(value::Value),
}

#[derive(Debug)]
pub struct TokenList {
    pub objects: Vec<Token>
}

#[derive(Debug)]
pub struct Tokenized {
    pub lines: Vec<TokenList>,
}