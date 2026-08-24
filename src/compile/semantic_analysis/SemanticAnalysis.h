#pragma once

#include "compile/ast_node.h"
#include "compile/value_type.h"
#include <unordered_map>

using IdentifierMap = std::unordered_map<std::string, ValueType>;
using Scopes = std::vector<IdentifierMap>;

class SemanticAnalysis {
    private:
        ASTNode& program;

    public:
        SemanticAnalysis(ASTNode& ast) : program(ast) {}

        static ValueType lookup_identifier_type(const std::string &name, const Scopes &scopes);

        static ValueType attach_identifier_type(Identifier &identifier, const Scopes &scopes);

        static void set_identifier_type(Identifier &identifier, ValueType type, Scopes &scopes);

        static ValueType get_cast_type(ValueType a, ValueType b);

        ValueType analyze_expression(ASTNode &node, Scopes &scopes);

        void analyze_let(LetDeclaration &let_declaration, Scopes &scopes);

        void analyze_set(SetDeclaration &set_declaration, Scopes &scopes);

        void analyze_program(Program &target, Scopes &scopes);

        void analyze();
};
