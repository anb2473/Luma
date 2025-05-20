use crate::parser_core::value;

enum Verb {
    Add,
    Return,
}

enum Token {
    Verb(Verb),
    Noun(value::Value),
}

struct TokenList {
    objects: Vec<>
}

struct Tokenized {
    lines: Vec<TokenList>,
}