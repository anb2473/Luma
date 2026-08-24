#include <compile/error.h>
#include <vector>
#include "compile/ast_node.h"
#include "compile/token.h"
#include <memory>
#include <utility>
#include "parser.h"

ASTNode Parser::parse_identifier() {
    Token identifier = tokens.read_token();

    if (identifier.type != TokenType::Identifier) {
        throw ParserError(
            "Invalid token type for identifier"
        );
    }

    return {
        Identifier{
            tokens.get_source(identifier)
        }
    };
}

ASTNode Parser::parse_num() {
    Token token = tokens.read_token();
    return {
        IntLiteral{
            std::stoi(tokens.get_source(token))
        }
    };
}

ASTNode Parser::parse_float() {
    Token token = tokens.read_token();
    return {
        FloatLiteral{
            std::stod(tokens.get_source(token))
        }
    };
}

ASTNode Parser::parse_bool() {
    Token token = tokens.read_token();
    if (
        token.type != TokenType::True &&
        token.type != TokenType::False
    ) {
        throw ParserError("Expected boolean");
    }

    return {
        BoolLiteral{
            token.type == TokenType::True
        }
    };
}

ASTNode Parser::parse_str() {
    Token token = tokens.read_token();
    return ASTNode{
        StringLiteral{
            tokens.get_source(token)
        }
    };
}

ASTNode Parser::parse_binary_expression(ASTNode left, ASTNode right, TokenType operation) {
    return {
        BinaryExpression{
            operation,
        std::make_unique<ASTNode>(std::move(left)),
            std::make_unique<ASTNode>(std::move(right))
        }
    };
}

int Parser::get_precedence(const TokenType token_type) {
    switch (token_type) {
        case TokenType::Mult:
        case TokenType::Div:
            return Precedence::Multiplicative;

        case TokenType::Add:
        case TokenType::Sub:
            return Precedence::Additive;

        case TokenType::IsLess:
        case TokenType::IsLessOrEq:
        case TokenType::IsGreater:
        case TokenType::IsGreaterOrEq:
            return Precedence::Conditional;

        case TokenType::IsEq:
        case TokenType::IsNotEq:
            return Precedence::Equality;

        default:
            return Invalid;
    }
}

ASTNode Parser::parse_expression(Precedence min_precedence) {
    ASTNode left = parse_value();
    while (true) {
        TokenType operation = tokens.peep_token().type;

        const int precedence = get_precedence(operation);

        if (
            precedence == Invalid ||
            precedence < min_precedence
        ) {
            break;
        }

        tokens.consume();

        ASTNode right = parse_expression(static_cast<Precedence>(precedence + 1));

        left = parse_binary_expression(
            std::move(left),
            std::move(right),
            operation
        );
    }
    return left;
}

ASTNode Parser::parse_value() {
    Token token = tokens.peep_token();
    switch (token.type) {
        case TokenType::String:
            return parse_str();
        case TokenType::Int:
            return parse_num();
        case TokenType::Float:
            return parse_float();
        case TokenType::Identifier: {
            return parse_identifier();
        }
        case TokenType::False:
        case TokenType::True:
            return parse_bool();
        default:
            throw ParserError("Invalid token for value");
    }
}

std::unique_ptr<ASTNode> Parser::parse_let() {
    ASTNode identifier = parse_identifier();
    tokens.expect(TokenType::Eq);
    ASTNode value = parse_expression();
    tokens.expect(TokenType::Semicolon);

    return std::make_unique<ASTNode>(
        ASTNode {
            LetDeclaration{
                std::make_unique<ASTNode>(std::move(identifier)),
                std::make_unique<ASTNode>(std::move(value))
            }
        }
    );
}


std::unique_ptr<ASTNode> Parser::parse_set() {
    ASTNode identifier = parse_identifier();
    tokens.expect(TokenType::Eq);

    ASTNode value = parse_expression();
    tokens.expect(TokenType::Semicolon);

    return std::make_unique<ASTNode>(
        ASTNode {
            SetDeclaration{
                std::make_unique<ASTNode>(std::move(identifier)),
                std::make_unique<ASTNode>(std::move(value))
            }
        }
    );
}

std::unique_ptr<ASTNode> Parser::parse_print() {
    ASTNode value = parse_expression();
    tokens.expect(TokenType::Semicolon);

    return std::make_unique<ASTNode>(
        ASTNode {
            PrintStatement{
                std::make_unique<ASTNode>(std::move(value))
            }
        }
    );
}

std::unique_ptr<ASTNode> Parser::parse_break() {
    tokens.expect(TokenType::Semicolon);

    return std::make_unique<ASTNode>(
        ASTNode {
            BreakStatement{}
        }
    );
}

std::unique_ptr<ASTNode> Parser::parse_continue() {
    tokens.expect(TokenType::Semicolon);

    return std::make_unique<ASTNode>(
        ASTNode {
            ContinueStatement{}
        }
    );
}

Program Parser::parse_scope() {
    Program program;
    while (!tokens.at_end()) {
        Token token = tokens.read_token();
        if (token.type == TokenType::CloseBrace) {
            return program;
        }
        switch (token.type)
        {
            case TokenType::Let:
                program.nodes.push_back(parse_let());
                break;
            case TokenType::Set:
                program.nodes.push_back(parse_set());
                break;
            case TokenType::Print:
                program.nodes.push_back(parse_print());
                break;
            case TokenType::If:
                program.nodes.push_back(parse_if());
                break;
            case TokenType::While:
                program.nodes.push_back(parse_while());
                break;
            case TokenType::Break:
                program.nodes.push_back(parse_break());
                break;
            case TokenType::Continue:
                program.nodes.push_back(parse_continue());
                break;
            default:
                throw ParserError("Invalid token");
        }
    }

    return program;
}

std::unique_ptr<ASTNode> Parser::parse_if() {
    ASTNode value = parse_expression();
    tokens.expect(TokenType::OpenBrace);

    Program program = parse_scope();

    Program else_program;
    if (!tokens.at_end() && tokens.peep_token().type == TokenType::Else) {
        tokens.consume();

        Token next = tokens.read_token();
        if (next.type == TokenType::If) {
            else_program.nodes.push_back(parse_if());
        } else if (next.type == TokenType::OpenBrace) {
            else_program = parse_scope();
        } else throw ParserError("Expected close brace");
    }
    return std::make_unique<ASTNode>(
        ASTNode {
            IfDeclaration{
                std::make_unique<ASTNode>(std::move(value)),
                std::make_unique<Program>(std::move(program)),
                std::make_unique<Program>(std::move(else_program))
            }
        }
    );
}


std::unique_ptr<ASTNode> Parser::parse_while() {
    ASTNode value = parse_expression();
    tokens.expect(TokenType::OpenBrace);

    Program program = parse_scope();

    return std::make_unique<ASTNode>(
            ASTNode {
                WhileDeclaration{
                    std::make_unique<ASTNode>(std::move(value)),
                    std::make_unique<Program>(std::move(program)),
                }
            }
        );
}

ASTNode Parser::parse() {
    return {
        parse_scope()
    };
}
