#include "TokenStream.h"
#include "compile/error.h"

bool TokenStream::at_end() const {
    return index >= size ||
           tokens[index].type == TokenType::EndOfFile;
}

Token TokenStream::read_token() {
    if (index >= size) {
        throw TokenStreamError("Unexpected end of input");
    }
    return tokens[index++];
}

void TokenStream::consume() {
    if (index >= size) {
        throw TokenStreamError("Unexpected end of input");
    }
    index++;
}

void TokenStream::expect(TokenType expected) {
    Token token = read_token();
    if (token.type != expected) {
        throw ParserError("Unexpected token");
    }
}

const Token& TokenStream::peep_token() {
    if (index >= size) {
        throw TokenStreamError("Unexpected end of input");
    }
    return tokens[index];
}

std::string TokenStream::get_source(const Token& token) {
    return source.substr(token.start, token.size);
}