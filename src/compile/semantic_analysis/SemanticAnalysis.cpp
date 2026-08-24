#include "SemanticAnalysis.h"
#include "compile/ast_node.h"
#include "compile/error.h"
#include "compile/value_type.h"
#include "compile/overloaded_visitor/overloaded_visitor.h"
#include <ranges>
#include <unordered_set>

static bool is_condition(const TokenType type) {
    switch (type) {
        case TokenType::IsEq:
        case TokenType::IsNotEq:
        case TokenType::IsLess:
        case TokenType::IsLessOrEq:
        case TokenType::IsGreater:
        case TokenType::IsGreaterOrEq:
            return true;
        default:
            return false;
    }
}

ValueType SemanticAnalysis::lookup_identifier_type(const std::string& name, const Scopes& scopes) {
    for (const auto & scope : std::views::reverse(scopes)) {
        auto identifier = scope.find(name);

        if (identifier == scope.end()) {
            continue;
        }
        return identifier->second;
    }
    throw SemanticAnalysisError(
        "Undefined identifier"
    );
}

ValueType SemanticAnalysis::attach_identifier_type(Identifier &identifier, const Scopes &scopes) {
    identifier.type = lookup_identifier_type(identifier.name, scopes);
    return identifier.type;
}

void SemanticAnalysis::set_identifier_type(Identifier& identifier, const ValueType type, Scopes& scopes) {
    const auto existing = scopes.back().find(identifier.name);

    if (existing != scopes.back().end()) {
        throw SemanticAnalysisError("Identifier redeclared");
    }

    scopes.back().insert({identifier.name, type});
}

ValueType SemanticAnalysis::get_cast_type(ValueType a, ValueType b) {
    if (a == ValueType::StrPtr || b == ValueType::StrPtr) {
        return ValueType::StrPtr;
    }

    if (a == ValueType::Float || b == ValueType::Float) {
        return ValueType::Float;
    }

    if (a == ValueType::Int || b == ValueType::Int) {
        return ValueType::Int;
    }

    if (a == ValueType::Bool || b == ValueType::Bool) {
        return ValueType::Bool;
    }

    throw BytecodeError("Invalid types");
}

ValueType SemanticAnalysis::analyze_expression(ASTNode& node, Scopes& scopes) {
    return std::visit(
        Overloaded(
            [](const FloatLiteral&)  {
                return ValueType::Float;
            },
            [](const IntLiteral&)  {
                return ValueType::Int;
            },
            [](const StringLiteral&)  {
                return ValueType::StrPtr;
            },
            [](const BoolLiteral&)  {
                return ValueType::Bool;
            },
            [&scopes](Identifier& identifier)  {
                return attach_identifier_type(identifier, scopes);
            },
            [&scopes, this](BinaryExpression& expression)  {
                ValueType left = analyze_expression(*expression.left, scopes);
                ValueType right = analyze_expression(*expression.right, scopes);
                expression.cast_type = get_cast_type(left, right);
                expression.return_type = is_condition(expression.operation) ?
                    ValueType::Bool : expression.cast_type;
                return expression.return_type;
            },
        [](const auto&) -> ValueType {
                throw SemanticAnalysisError("Unexpected type");
            }
        ),
        node.data
    );
}

void SemanticAnalysis::analyze_let(LetDeclaration& let_declaration, Scopes& scopes) {
    const ValueType expression_type = analyze_expression(*let_declaration.value, scopes);

    set_identifier_type(
        std::get<Identifier>((*let_declaration.identifier).data),
        expression_type,
        scopes
    );

    let_declaration.cast_type = expression_type;
}

void SemanticAnalysis::analyze_set(SetDeclaration& set_declaration, Scopes& scopes) {
    const ValueType expression_type = analyze_expression(*set_declaration.value, scopes);

    ValueType type = lookup_identifier_type(
        std::get<Identifier>((*set_declaration.identifier).data).name,
        scopes
    );

    if (expression_type != type) {
        throw SemanticAnalysisError("Identifier reassigned");
    }

    set_declaration.cast_type = type;
}

void SemanticAnalysis::analyze_program(Program& target, Scopes& scopes) {
    scopes.emplace_back();
    for (auto& node : target.nodes) {
        std::visit(
            Overloaded(
                [this, &scopes](LetDeclaration& let_declaration)  {
                    analyze_let(let_declaration, scopes);
                },
                [this, &scopes](SetDeclaration& set_declaration)  {
                    analyze_set(set_declaration, scopes);
                },
                [this, &scopes](WhileDeclaration& while_declaration)  {
                    ValueType condition_type = analyze_expression(*while_declaration.value, scopes);

                    if (condition_type != ValueType::Bool) {
                        throw SemanticAnalysisError(
                            "While condition must be boolean"
                        );
                    }

                    while_declaration.cast_type = condition_type;

                    analyze_program(*while_declaration.program, scopes);
                },
                [this, &scopes](IfDeclaration& if_declaration)  {
                    ValueType condition_type = analyze_expression(*if_declaration.value, scopes);

                    if (condition_type != ValueType::Bool) {
                        throw SemanticAnalysisError(
                            "If condition must be boolean"
                        );
                    }

                    if_declaration.cast_type = condition_type;

                    analyze_program(*if_declaration.program, scopes);
                    analyze_program(*if_declaration.else_program, scopes);
                },
                [this, &scopes](PrintStatement& print_statement)  {
                    ValueType type = analyze_expression(*print_statement.value, scopes);
                    print_statement.cast_type = type;
                },
                [](BreakStatement&) {},
                [](ContinueStatement&)  {},
                [](const auto&) {
                    throw SemanticAnalysisError("Expected expression");
                }
            ),
            node->data
        );
    }
    scopes.pop_back();
}

void SemanticAnalysis::analyze() {
    Scopes scopes;
    analyze_program(std::get<Program>(program.data), scopes);
}