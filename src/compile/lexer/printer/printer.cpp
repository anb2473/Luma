#include <iostream>
#include "printer.h"

void print_tokens(
    std::ostream& out,
    const std::vector<Token>& tokens,
    const std::string& source
) {
    for (const Token& token : tokens) {
        out << "type=";

        switch (token.type) {
            case TokenType::Let:
                out << "Let";
                break;

            case TokenType::If:
                out << "If";
                break;

            case TokenType::Else:
                out << "Else";
                break;

            case TokenType::Identifier:
                out << "Identifier";
                break;

            case TokenType::Eq:
                out << "Eq";
                break;

            case TokenType::Int:
                out << "Int";
                break;

            case TokenType::Float:
                out << "Float";
                break;

            case TokenType::True:
                out << "True";
                break;

            case TokenType::False:
                out << "False";
                break;

            case TokenType::String:
                out << "String";
                break;

            case TokenType::Add:
                out << "Add";
                break;

            case TokenType::Sub:
                out << "Sub";
                break;

            case TokenType::Mult:
                out << "Mult";
                break;

            case TokenType::Div:
                out << "Div";
                break;

            case TokenType::IsEq:
                out << "IsEq";
                break;

            case TokenType::IsNotEq:
                out << "IsNotEq";
                break;

            case TokenType::IsGreater:
                out << "IsGreater";
                break;

            case TokenType::IsLess:
                out << "IsLess";
                break;

            case TokenType::IsLessOrEq:
                out << "IsLessOrEq";
                break;

            case TokenType::IsGreaterOrEq:
                out << "IsGreaterOrEq";
                break;

            case TokenType::Semicolon:
                out << "Semicolon";
                break;

            case TokenType::EndOfFile:
                out << "EndOfFile";
                break;

            case TokenType::Dot:
                out << "Dot";
                break;

            case TokenType::Print:
                out << "Print";
                break;

            case TokenType::OpenBrace:
                out << "OpenBrace";
                break;

            case TokenType::CloseBrace:
                out << "CloseBrace";
                break;
        }

        out
            << " val=\""
            << source.substr(token.start, token.size)
            << "\"\n";
    }
}