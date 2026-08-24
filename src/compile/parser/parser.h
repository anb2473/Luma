#pragma once

#include <vector>
#include "compile/ast_node.h"
#include "compile/token.h"
#include "token_stream/TokenStream.h"

enum Precedence {
    Invalid = -1,
    Equality = 0,
    Conditional = 1,
    Additive = 2,
    Multiplicative = 3,
};

class Parser {
    private:
        TokenStream tokens;

        std::unique_ptr<ASTNode> parse_let();
        std::unique_ptr<ASTNode> parse_set();
        std::unique_ptr<ASTNode> parse_print();
        std::unique_ptr<ASTNode> parse_if();
        std::unique_ptr<ASTNode> parse_continue();
        std::unique_ptr<ASTNode> parse_break();
        std::unique_ptr<ASTNode> parse_while();

        static ASTNode parse_binary_expression(ASTNode left, ASTNode right, TokenType operation);

        ASTNode parse_expression(Precedence min_precedence = Equality);
        ASTNode parse_value();

        ASTNode parse_float();
        ASTNode parse_str();
        ASTNode parse_bool();
        ASTNode parse_identifier();
        ASTNode parse_num();

        Program parse_scope();

        static int get_precedence(TokenType token_type);

    public:
        Parser(std::vector<Token> tokens, const std::string& source)
            : tokens(std::move(tokens), source) {}
        ASTNode parse();
};
