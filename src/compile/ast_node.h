#pragma once

#include <string>
#include <variant>
#include <vector>
#include "compile/token.h"
#include "compile/value_type.h"

struct ASTNode;

struct Program {
    std::vector<std::unique_ptr<ASTNode>> nodes;
};

struct LetDeclaration {
    std::unique_ptr<ASTNode> identifier;
    std::unique_ptr<ASTNode> value;
    ValueType cast_type;
};

struct SetDeclaration {
    std::unique_ptr<ASTNode> identifier;
    std::unique_ptr<ASTNode> value;
    ValueType cast_type;
};

struct IfDeclaration {
    std::unique_ptr<ASTNode> value;
    std::unique_ptr<Program> program;
    std::unique_ptr<Program> else_program;
    ValueType cast_type;
};

struct WhileDeclaration {
    std::unique_ptr<ASTNode> value;
    std::unique_ptr<Program> program;
    ValueType cast_type;
};

struct PrintStatement {
    std::unique_ptr<ASTNode> value;
    ValueType cast_type;
};

struct SleepStatement {
    std::unique_ptr<ASTNode> value;
    ValueType cast_type;
};

struct Identifier {
    std::string name;
    mutable ValueType type;
};

struct IntLiteral {
    int value;
};

struct FloatLiteral {
    double value;
};

struct StringLiteral {
    std::string value;
};

struct BoolLiteral {
    bool value;
};

struct BreakStatement {};

struct ContinueStatement {};

struct BinaryExpression {
    TokenType operation;
    std::unique_ptr<ASTNode> left;
    std::unique_ptr<ASTNode> right;
    ValueType cast_type;
    ValueType return_type;
};

struct UnaryExpression {
    TokenType operation;
    std::unique_ptr<ASTNode> right;
    ValueType cast_type;
    ValueType return_type;
};

using ASTData = std::variant<
    Program,
    LetDeclaration,
    Identifier,
    IntLiteral,
    FloatLiteral,
    StringLiteral,
    PrintStatement,
    BoolLiteral,
    IfDeclaration,
    WhileDeclaration,
    BreakStatement,
    ContinueStatement,
    SetDeclaration,
    BinaryExpression,
    SleepStatement,
    UnaryExpression
>;

struct ASTNode {
    ASTData data;
};
