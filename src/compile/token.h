#pragma once

#include <cstddef>

enum class TokenType {
    Let,
    Identifier,
    Eq,
    Int,
    Float,
    Semicolon,
    EndOfFile,
    String,
    Dot,
    Add,
    Sub,
    Div,
    Mult,
    Print,
    False,
    True,
    IsEq,
    IsLess,
    IsGreater,
    IsGreaterOrEq,
    IsLessOrEq,
    If,
    OpenBrace,
    CloseBrace,
    IsNotEq,
    Else,
    While,
    Break,
    Continue,
    Set,
    Sleep,
    Mod,
    OpenParen,
    CloseParen
};

struct Token {
    TokenType type;
    size_t start;
    size_t size;
};
