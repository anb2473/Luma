#include <string>
#include <vector>
#include "compile/error.h"
#include "compile/token.h"
#include "lexer.h"
#include <unordered_map>
#include <optional>

static const std::unordered_map<std::string_view, TokenType> keywords = {
    {"let", TokenType::Let},
    {"set", TokenType::Set},
    {"print", TokenType::Print},
    {"true", TokenType::True},
    {"false", TokenType::False},
    {"if", TokenType::If},
    {"else", TokenType::Else},
    {"while", TokenType::While},
    {"break", TokenType::Break},
    {"continue", TokenType::Continue},
};

static const std::unordered_map<int, TokenType> single_char_tokens = {
    {';', TokenType::Semicolon},
    {'{', TokenType::OpenBrace},
    {'}', TokenType::CloseBrace},
    {'+', TokenType::Add},
    {'-', TokenType::Sub},
    {'/', TokenType::Div},
    {'*', TokenType::Mult},
};

int Lexer::peek(size_t forwards) const {
    if (position + forwards >= source.size()) {
        return EOF;
    }
    return static_cast<unsigned char>(source[position + forwards]);
}

int Lexer::read() {
    if (position >= source.size()) {
        return EOF;
    }
    return static_cast<unsigned char>(source[position++]);
}

void Lexer::consume(const size_t n) {
    position += n;
}

Token Lexer::read_n(TokenType type, size_t n) {
    const size_t start = position;
    consume(n);
    return {
        type,
        start,
        n
    };
}

Token Lexer::read_str() {
    const int delimiter = read();
    const size_t start = position;
    int ch;
    while ((ch = read()) != delimiter) {
        if (ch == EOF) {
            throw LexerError("Missing delimiter");
        }
    }

    return {
        TokenType::String,
        start,
        position - start - 1
    };
}

bool Lexer::is_num(const int ch) {
    return std::isdigit(ch) || ch == '.';
}

Token Lexer::read_num() {
    const size_t start = position;

    int ch;
    bool is_float = false;
    while (is_num(ch = peek())) {
        if (ch == '.') {
            if (is_float) {
                throw LexerError("Multiple floating points");
            }
            is_float = true;
        }

        consume();
    }

    const size_t size = position - start;

    if (is_float && size == 1) {
        throw LexerError("Invalid float");
    }

    return {
        .type = is_float ? TokenType::Float : TokenType::Int,
        .start = start,
        .size = size
    };
}

bool Lexer::is_word(const int ch) {
    return std::isalnum(ch) || ch == '_';
}

TokenType Lexer::get_word_type(const std::string_view& word) {
    if (const auto keyword = keywords.find(word); keyword != keywords.end()) {
        return keyword->second;
    }

    return  TokenType::Identifier;
}

Token Lexer::read_word() {
    const size_t start = position;

    while (is_word(peek())) {
        consume();
    }

    const size_t size = position - start;

    const std::string_view word(source.data() + start, size);

    const TokenType type = get_word_type(word);

    return {
        .type = type,
        .start = start,
        .size = size
    };
}

std::optional<Token> Lexer::read_special_token(int ch) {
    switch (ch) {
        case '"':
        case '\'':
            return read_str();
        case '=': {
            if (peek(1) == '=') {
                return read_n(TokenType::IsEq, 2);
            }
            return read_n(TokenType::Eq);
        }
        case '>': {
            if (peek(1) == '=') {
                return read_n(TokenType::IsGreaterOrEq, 2);
            }
            return read_n(TokenType::IsGreater);
        }
        case '!': {
            if (peek(1)  == '=') {
                return read_n(TokenType::IsNotEq, 2);
            }
            return std::nullopt;
        }
        case '<': {
            if (peek(1) == '=') {
                return read_n(TokenType::IsLessOrEq, 2);
            }
            return read_n(TokenType::IsLess);
        }
        default:
            return std::nullopt;
    }
}

Token Lexer::read_token() {
    int ch;

    while (std::isspace(ch = peek())) {
        consume();
    }

    if (ch == EOF) {
        return read_n(TokenType::EndOfFile, 0);
    }

    if (is_num(ch)) {
        return read_num();
    }

    if (is_word(ch)) {
        return read_word();
    }

    if (auto it = single_char_tokens.find(ch); it != single_char_tokens.end()) {
        return read_n(it->second);
    }

    if (auto token = read_special_token(ch)) {
        return *token;
    }

    throw LexerError(
        "Invalid Token"
    );
}

std::vector<Token> Lexer::tokenize() {
    std::vector<Token> tokens;

    Token current_token = read_token();
    while (current_token.type != TokenType::EndOfFile) {
        tokens.push_back(current_token);
        current_token = read_token();
    }

    return tokens;
}
