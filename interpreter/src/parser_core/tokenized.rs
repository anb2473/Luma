use crate::parser_core::value;

#[derive(Clone, Debug)]
pub enum Verb {
    Add,
    Sub,
    Mult,
    Div,
    Set,
}

#[derive(Clone, Debug)]
pub enum Suffix {
    Set,
}

#[derive(Debug)]
pub enum Token {
    Verb(Verb),
    Noun(value::Value),
}

#[derive(Debug)]
pub struct TokenList {
    pub objects: Vec<Token>,
    pub suffix: Option<Suffix>,
}

#[derive(Debug)]
pub struct Tokenized {
    pub lines: Vec<TokenList>,
}