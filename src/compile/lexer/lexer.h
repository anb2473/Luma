#pragma once

#include <vector>
#include "compile/token.h"

class Lexer {
    private:
        const std::string& source;
        size_t position = 0;

        [[nodiscard]] int peek(size_t forwards = 0) const;
        [[nodiscard]] int read();
        void consume(size_t n = 1);

        Token read_n(TokenType type, size_t n = 1);
        Token read_str();

        static bool is_num(int ch);
        Token read_num();

        static bool is_word(int ch);
        static TokenType get_word_type(const std::string_view& word);
        Token read_word();

        std::optional<Token> read_special_token(int ch);
        Token read_token();

    public:
        explicit Lexer(const std::string& source)
            : source(source) {}
        std::vector<Token> tokenize();
};
