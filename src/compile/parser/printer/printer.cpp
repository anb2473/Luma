#include <iostream>
#include "compile/ast_node.h"
#include "compile/overloaded_visitor/overloaded_visitor.h"

static const char* value_type_name(ValueType type) {
    switch (type) {
        case ValueType::Int:     return "Int";
        case ValueType::Float:   return "Float";
        case ValueType::Bool:    return "Bool";
        case ValueType::StrPtr:  return "StrPtr";
        default:                 return "???";
    }
}

static const char* token_type_name(TokenType type) {
    switch (type) {
        case TokenType::Add:           return "Add";
        case TokenType::Sub:           return "Sub";
        case TokenType::Mult:          return "Mult";
        case TokenType::Div:           return "Div";
        case TokenType::IsEq:          return "IsEq";
        case TokenType::IsNotEq:       return "IsNotEq";
        case TokenType::IsLess:        return "IsLess";
        case TokenType::IsLessOrEq:    return "IsLessOrEq";
        case TokenType::IsGreater:     return "IsGreater";
        case TokenType::IsGreaterOrEq: return "IsGreaterOrEq";
        default:                       return "???";
    }
}

static void print_types(
    std::ostream& out,
    ValueType return_type,
    ValueType cast_type
) {
    out << " [return_type="
        << value_type_name(return_type)
        << ", cast_type="
        << value_type_name(cast_type)
        << "]";
}

void print_ast_node(
    std::ostream& out,
    const ASTNode& node,
    int indent
) {
    const std::string padding(indent, ' ');

    std::visit(
        Overloaded(

            [&](const Program& program) {
                out << padding << "Program\n";

                for (const auto& statement : program.nodes) {
                    print_ast_node(
                        out,
                        *statement,
                        indent + 2
                    );
                }
            },

            [&](const LetDeclaration& let) {
                out << padding << "LetDeclaration\n";

                out << padding << "  Identifier:\n";
                print_ast_node(
                    out,
                    *let.identifier,
                    indent + 4
                );

                out << padding << "  Value:\n";
                print_ast_node(
                    out,
                    *let.value,
                    indent + 4
                );
            },

            [&](const SetDeclaration& set) {
                out << padding << "SetDeclaration\n";

                out << padding << "  Identifier:\n";
                print_ast_node(
                    out,
                    *set.identifier,
                    indent + 4
                );

                out << padding << "  Value:\n";
                print_ast_node(
                    out,
                    *set.value,
                    indent + 4
                );
            },

            [&](const IfDeclaration& if_statement) {
                out << padding << "IfDeclaration\n";

                out << padding << "  Condition:\n";
                print_ast_node(
                    out,
                    *if_statement.value,
                    indent + 4
                );

                out << padding << "  Program:\n";

                for (const auto& statement :
                     if_statement.program->nodes) {
                    print_ast_node(
                        out,
                        *statement,
                        indent + 4
                    );
                }

                if (if_statement.else_program &&
                    !if_statement.else_program->nodes.empty()) {

                    out << padding << "  ElseProgram:\n";

                    for (const auto& statement :
                         if_statement.else_program->nodes) {
                        print_ast_node(
                            out,
                            *statement,
                            indent + 4
                        );
                    }
                }
            },

            [&](const WhileDeclaration& while_statement) {
                out << padding << "WhileDeclaration\n";

                out << padding << "  Condition:\n";
                print_ast_node(
                    out,
                    *while_statement.value,
                    indent + 4
                );

                out << padding << "  Program:\n";

                for (const auto& statement :
                     while_statement.program->nodes) {
                    print_ast_node(
                        out,
                        *statement,
                        indent + 4
                    );
                }
            },

            [&](const PrintStatement& statement) {
                out << padding << "PrintStatement\n";

                out << padding << "  Value:\n";
                print_ast_node(
                    out,
                    *statement.value,
                    indent + 4
                );
            },

            [&](const BreakStatement&) {
                out << padding << "BreakStatement\n";
            },

            [&](const ContinueStatement&) {
                out << padding << "ContinueStatement\n";
            },

            [&](const Identifier& identifier) {
                out << padding
                    << "Identifier: "
                    << identifier.name
                    << " [type="
                    << value_type_name(identifier.type)
                    << "]\n";
            },

            [&](const IntLiteral& value) {
                out << padding
                    << "IntLiteral: "
                    << value.value
                    << "\n";
            },

            [&](const FloatLiteral& value) {
                out << padding
                    << "FloatLiteral: "
                    << value.value
                    << "\n";
            },

            [&](const BoolLiteral& value) {
                out << padding
                    << "BoolLiteral: "
                    << (value.value ? "true" : "false")
                    << "\n";
            },

            [&](const StringLiteral& value) {
                out << padding
                    << "StringLiteral: \""
                    << value.value
                    << "\"\n";
            },

            [&](const BinaryExpression& expression) {
                out << padding
                    << "BinaryExpression: "
                    << token_type_name(expression.operation);

                print_types(
                    out,
                    expression.return_type,
                    expression.cast_type
                );

                out << "\n";

                out << padding << "  Left:\n";
                print_ast_node(
                    out,
                    *expression.left,
                    indent + 4
                );

                out << padding << "  Right:\n";
                print_ast_node(
                    out,
                    *expression.right,
                    indent + 4
                );
            }
        ),
        node.data
    );
}

void print_ast(
    std::ostream& out,
    const ASTNode& ast
) {
    out << "AST\n";
    print_ast_node(out, ast, 2);
}