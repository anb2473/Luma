#pragma once

#include "compile/token.h"
#include <vector>

class TokenStream {
    private:
        size_t index = 0;
        size_t size;
        std::vector<Token> tokens;
        std::string source;
    public:
        TokenStream(std::vector<Token> tokens, const std::string& source)
            : size(tokens.size()), tokens(std::move(tokens)), source(source) {}
        const Token& peep_token();
        Token read_token();

        void consume();

        void expect(TokenType expected);

        [[nodiscard]] bool at_end() const;
        std::string get_source(const Token& token);
};

