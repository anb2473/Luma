#include <vector>
#include "compile/ast_node.h"
#include "compile/error.h"
#include <cstdint>
#include <string>
#include "bytecode.h"
#include "codes.h"
#include <iostream>
#include <ranges>

#include "compile/overloaded_visitor/overloaded_visitor.h"

ValueType Compiler::cast_int_to(ValueType superior_type, const IntLiteral& int_literal) {
    switch (superior_type) {
        case ValueType::Float:
           writer.write_u8(static_cast<uint8_t>(Opcode::PushFloat));
           writer.write_f64(static_cast<double>(int_literal.value));
           return ValueType::Float;
        case ValueType::Int:
            writer.write_u8(static_cast<uint8_t>(Opcode::PushInt));
            writer.write_u32(int_literal.value);
           return ValueType::Int;
       default:
           throw BytecodeError("Invalid type cast");
    }
}

ValueType Compiler::cast_float_to(ValueType superior_type, const FloatLiteral& float_literal) {
    switch (superior_type) {
        case ValueType::Float:
           writer.write_u8(static_cast<uint8_t>(Opcode::PushFloat));
           writer.write_f64(float_literal.value);
           return ValueType::Float;
       default:
           throw BytecodeError("Invalid type cast");
    }
}

ValueType Compiler::cast_bool_to(ValueType superior_type, const BoolLiteral& bool_literal) {
    switch (superior_type) {
        case ValueType::Int:
            writer.write_u8(static_cast<uint8_t>(Opcode::PushInt));
            writer.write_u32(static_cast<uint32_t>(bool_literal.value));
            return ValueType::Int;
        case ValueType::Bool:
            writer.write_u8(static_cast<uint8_t>(Opcode::PushBool));
            writer.write_u8(bool_literal.value);
            return ValueType::Bool;
       default:
           throw BytecodeError("Invalid type cast");
    }
}

ValueType Compiler::cast_str_to(ValueType superior_type, const StringLiteral& string_literal) {
    switch (superior_type) {
        case ValueType::StrPtr: {
            writer.write_u8(static_cast<uint8_t>(Opcode::PushConst));
            if (constants.contains(string_literal.value)) {
                auto existing = constants.at(string_literal.value);
                writer.write_u32(existing.pos);
                return ValueType::StrPtr;
            }

            uint32_t pos = static_cast<uint32_t>(writer.front_size());

            writer.write_u32(pos);

            constants.emplace(
                string_literal.value,
                Constant{
                    ConstantType::String,
                    string_literal.value,
                    pos
                }
            );

            if (string_literal.value.size() > UINT16_MAX) {
                throw BytecodeError("String literal exceeds maximum size");
            }
            writer.write_front_string(string_literal.value);

            return ValueType::StrPtr;
        }
        default:
           throw BytecodeError("Invalid type cast");
    }
}

ValueType Compiler::cast_identifier_to(
    ValueType superior_type,
    const Identifier& identifier,
    ValueType identifier_type
 ){
    int offset = -1;
    for (auto it = scopes.rbegin(); it != scopes.rend(); it++) {
        auto symbol = it->find(identifier.name);
        if (symbol != it->end()) {
            offset = symbol->second.offset;
            break;
        }
    }
    if (offset < 0) {
        throw BytecodeError("Identifier not found");
    }

    switch (identifier_type) {
        case ValueType::StrPtr:
            writer.write_u8(static_cast<uint8_t>(Opcode::LoadStrPtr));
            writer.write_u16(offset);
            switch (superior_type) {
                case ValueType::StrPtr:
                    return ValueType::StrPtr;
                default:
                    throw BytecodeError("Invalid cast");
            }
        case ValueType::Int:
            writer.write_u8(static_cast<uint8_t>(Opcode::LoadInt));
            writer.write_u16(offset);
            switch (superior_type) {
                case ValueType::StrPtr:
                    writer.write_u8(static_cast<uint8_t>(Opcode::IntToStrPtr));
                    return ValueType::StrPtr;
                case ValueType::Float:
                    writer.write_u8(static_cast<uint8_t>(Opcode::IntToFloat));
                    return ValueType::Float;
                case ValueType::Int:
                    return ValueType::Int;
                default:
                    throw BytecodeError("Invalid cast");
            }
        case ValueType::Float:
            writer.write_u8(static_cast<uint8_t>(Opcode::LoadFloat));
            writer.write_u16(offset);
            switch (superior_type) {
                case ValueType::StrPtr:
                    writer.write_u8(static_cast<uint8_t>(Opcode::FloatToStrPtr));
                    return ValueType::StrPtr;
                case ValueType::Float:
                    return ValueType::Float;
                default:
                    throw BytecodeError("Invalid cast");
            }
        case ValueType::Bool:
            writer.write_u8(static_cast<uint8_t>(Opcode::LoadBool));
            writer.write_u16(offset);
            switch (superior_type) {
                case ValueType::Bool:
                    return ValueType::Bool;;
                default:
                    throw BytecodeError("Invalid cast");
            }
        default:
            throw BytecodeError("Invalid cast");
    }
}

ValueType Compiler::cast(ValueType superior_type, const ASTNode& node) {
    return std::visit(
        Overloaded(
            [this, &superior_type](const IntLiteral& int_literal) {
                return cast_int_to(
                    superior_type,
                    int_literal
                );
            },
            [this, &superior_type](const FloatLiteral& float_literal) {
                return cast_float_to(
                    superior_type,
                    float_literal
                );
            },
            [this, &superior_type](const StringLiteral& string_literal) {
                return cast_str_to(
                    superior_type,
                    string_literal
                );
            },
            [this, &superior_type](const Identifier& identifier) {
                return cast_identifier_to(
                    superior_type,
                    identifier,
                    identifier.type
                );
            },
            [this, &superior_type](const BoolLiteral& bool_literal) {
                return cast_bool_to(
                    superior_type,
                    bool_literal
                );
            },
            [](auto&) -> ValueType {
                throw BytecodeError("Invalid type cast");
            }
        ),
        node.data
    );
}

ValueType Compiler::cast_binary_expression_to(const BinaryExpression& binary_expression, const ValueType cast_type) {
    switch (binary_expression.cast_type) {
        case ValueType::Int:
            switch (cast_type) {
                case ValueType::Int:
                    return ValueType::Int;
                    break;
                case ValueType::Float:
                    writer.write_u8(static_cast<uint8_t>(Opcode::IntToFloat));
                    return ValueType::Float;
                    break;
                case ValueType::StrPtr:
                    writer.write_u8(static_cast<uint8_t>(Opcode::IntToStrPtr));
                    return ValueType::StrPtr;
                    break;
                default:
                    throw BytecodeError("Invalid cast");
            }
        case ValueType::Float:
            switch (cast_type) {
                case ValueType::Int:
                case ValueType::Float:
                    return ValueType::Float;
                    break;
                case ValueType::StrPtr:
                    writer.write_u8(static_cast<uint8_t>(Opcode::FloatToStrPtr));
                    return ValueType::StrPtr;
                    break;
                default:
                    throw BytecodeError("Invalid cast");
            }
        case ValueType::StrPtr:
            switch (cast_type) {
                case ValueType::Int:
                case ValueType::Float:
                case ValueType::StrPtr:
                    return ValueType::StrPtr;
                    break;
                default:
                    throw BytecodeError("Invalid cast");
            }
        case ValueType::Bool:
            switch (cast_type) {
                case ValueType::Bool:
                    return ValueType::Bool;
                    break;
                default:
                    throw BytecodeError("Invalid cast");
            }
    }
    throw BytecodeError("Invalid type cast");
}

ValueType Compiler::compile_root(const ASTNode& root, ValueType cast_type) {
    return std::visit(
        Overloaded(
        [this](const UnaryExpression& unary_expression) {
                    compile_root(*unary_expression.right, unary_expression.cast_type);
                    switch (unary_expression.operation) {
                        case TokenType::Sub: {
                            Opcode opcode;
                            switch (unary_expression.cast_type) {
                                case ValueType::Int:
                                    opcode = Opcode::SubUnaryInt;
                                    break;
                                case ValueType::Float:
                                    opcode = Opcode::SubUnaryFloat;
                                    break;
                                default:
                                    throw BytecodeError("Invalid type");
                            }

                            writer.write_u8(static_cast<uint8_t>(opcode));
                            return ValueType::Bool;
                        }
                        default:
                            throw BytecodeError("Invalid type");
                    }
                },
            [this, cast_type](const BinaryExpression& binary_expression) {
                compile_root(*binary_expression.left, binary_expression.cast_type);
                compile_root(*binary_expression.right, binary_expression.cast_type);

                switch (binary_expression.operation) {
                    case TokenType::IsEq: {
                        Opcode opcode;
                        switch (binary_expression.cast_type) {
                            case ValueType::Int:
                                opcode = Opcode::IsEqInt;
                                break;
                            case ValueType::Float:
                                opcode = Opcode::IsEqFloat;
                                break;
                            case ValueType::Bool:
                                opcode = Opcode::IsEqBool;
                                break;
                            default:
                                throw BytecodeError("Invalid type");
                        }

                        writer.write_u8(static_cast<uint8_t>(opcode));
                        cast_binary_expression_to(binary_expression, cast_type);
                        return ValueType::Bool;
                    }
                    case TokenType::IsNotEq: {
                        Opcode opcode;
                        switch (binary_expression.cast_type) {
                            case ValueType::Int:
                                opcode = Opcode::IsNotEqInt;
                                break;
                            case ValueType::Float:
                                opcode = Opcode::IsNotEqFloat;
                                break;
                            case ValueType::Bool:
                                opcode = Opcode::IsNotEqBool;
                                break;
                            default:
                                throw BytecodeError("Invalid type");
                        }

                        writer.write_u8(static_cast<uint8_t>(opcode));
                        cast_binary_expression_to(binary_expression, cast_type);
                        return ValueType::Bool;
                    }
                    case TokenType::IsGreater: {
                        Opcode opcode;
                        switch (binary_expression.cast_type) {
                            case ValueType::Int:
                                opcode = Opcode::IsGreaterInt;
                                break;
                            case ValueType::Float:
                                opcode = Opcode::IsGreaterFloat;
                                break;
                            default:
                                throw BytecodeError("Invalid type");
                        }

                        writer.write_u8(static_cast<uint8_t>(opcode));
                        cast_binary_expression_to(binary_expression, cast_type);
                        return ValueType::Bool;
                    }
                    case TokenType::IsLess: {
                        Opcode opcode;
                        switch (binary_expression.cast_type) {
                            case ValueType::Int:
                                opcode = Opcode::IsLessInt;
                                break;
                            case ValueType::Float:
                                opcode = Opcode::IsLessFloat;
                                break;
                            default:
                                throw BytecodeError("Invalid type");
                        }

                        writer.write_u8(static_cast<uint8_t>(opcode));
                        cast_binary_expression_to(binary_expression, cast_type);
                        return ValueType::Bool;
                    }
                    case TokenType::IsLessOrEq: {
                        Opcode opcode;
                        switch (binary_expression.cast_type) {
                            case ValueType::Int:
                                opcode = Opcode::IsLessOrEqInt;
                                break;
                            case ValueType::Float:
                                opcode = Opcode::IsLessOrEqFloat;
                                break;
                            default:
                                throw BytecodeError("Invalid type");
                        }

                        writer.write_u8(static_cast<uint8_t>(opcode));
                        cast_binary_expression_to(binary_expression, cast_type);
                        return ValueType::Bool;
                    }
                    case TokenType::IsGreaterOrEq: {
                        Opcode opcode;
                        switch (binary_expression.cast_type) {
                            case ValueType::Int:
                                opcode = Opcode::IsGreaterOrEqInt;
                                break;
                            case ValueType::Float:
                                opcode = Opcode::IsGreaterOrEqFloat;
                                break;
                            default:
                                throw BytecodeError("Invalid type");
                        }

                        writer.write_u8(static_cast<uint8_t>(opcode));
                        cast_binary_expression_to(binary_expression, cast_type);
                        return ValueType::Bool;
                    }
                    case TokenType::Add: {
                        Opcode opcode;
                        switch (binary_expression.cast_type) {
                            case ValueType::Int:
                                opcode = Opcode::AddInt;
                                break;
                            case ValueType::Float:
                                opcode = Opcode::AddFloat;
                                break;
                            case ValueType::StrPtr:
                                opcode = Opcode::AddStrPtr;
                                break;
                            default:
                                throw BytecodeError("Invalid type");
                        }

                        writer.write_u8(static_cast<uint8_t>(opcode));
                        return cast_binary_expression_to(binary_expression, cast_type);
                    }
                    case TokenType::Div: {
                        Opcode opcode;
                        switch (binary_expression.cast_type) {
                            case ValueType::Int:
                                opcode = Opcode::DivInt;
                                break;
                            case ValueType::Float:
                                opcode = Opcode::DivFloat;
                                break;
                            default:
                                throw BytecodeError("Invalid type");
                        }

                        writer.write_u8(static_cast<uint8_t>(opcode));
                        return cast_binary_expression_to(binary_expression, cast_type);
                    }
                    case TokenType::Mult: {
                        Opcode opcode;
                        switch (binary_expression.cast_type) {
                            case ValueType::Int:
                                opcode = Opcode::MultInt;
                                break;
                            case ValueType::Float:
                                opcode = Opcode::MultFloat;
                                break;
                            default:
                                throw BytecodeError("Invalid type");
                        }

                        writer.write_u8(static_cast<uint8_t>(opcode));
                        return cast_binary_expression_to(binary_expression, cast_type);
                    }
                    case TokenType::Sub: {
                        Opcode opcode;
                        switch (binary_expression.cast_type) {
                            case ValueType::Int:
                                opcode = Opcode::SubInt;
                                break;
                            case ValueType::Float:
                                opcode = Opcode::SubFloat;
                                break;
                            default:
                                throw BytecodeError("Invalid type");
                        }

                        writer.write_u8(static_cast<uint8_t>(opcode));
                        return cast_binary_expression_to(binary_expression, cast_type);
                    }
                    default:
                        throw BytecodeError("Invalid operation");
                }

            },
            [this, &cast_type, &root](auto&) {
                return cast(cast_type, root);
            }
        ),
        root.data
    );
}

void Compiler::clean_scope(const size_t start_scope) {
    if (start_scope > scopes.size()) {
        throw BytecodeError("Invalid start scope for cleanup");
    }
    for (size_t scope = scopes.size(); scope-- > start_scope;) {
        for (const auto &symbol: scopes.at(scope) | std::views::values) {
            switch (symbol.type) {
                case ValueType::StrPtr:
                    writer.write_u8(static_cast<uint8_t>(Opcode::DelStrPtr));
                    break;
                default:
                    writer.write_u8(static_cast<uint8_t>(Opcode::PopStack));
            }
        }
    }
}

void Compiler::compile_program(std::vector<LoopContext>& loop_context, Program& program) {
    scopes.emplace_back();

    for (const std::unique_ptr<ASTNode>& node : program.nodes) {
        std::visit(
            Overloaded(
                [this](SetDeclaration& set_declaration) {
                    ValueType type = compile_root(
                        *set_declaration.value,
                        set_declaration.cast_type
                    );

                    if (type == ValueType::StrPtr) {
                        throw BytecodeError("Cannot reassign string pointer");
                    }

                    const Identifier& identifier =
                        std::get<Identifier>(
                            set_declaration.identifier->data
                        );

                    uint16_t offset;
                    bool found = false;

                    for (auto scope = scopes.rbegin(); scope != scopes.rend(); ++scope) {
                        if (auto existing = scope->find(identifier.name); existing != scope->end()) {
                            if (
                                existing->second.type != type
                                ) {
                                throw BytecodeError("Cannot reassign type");
                            }
                            offset = scope->at(identifier.name).offset;
                            found = true;
                            break;
                        }
                    }

                    if (!found) {
                        throw BytecodeError("Invalid identifier");
                    }

                    Opcode opcode;

                    switch (type) {
                        case ValueType::Int:
                            opcode = Opcode::StoreInt;
                            break;

                        case ValueType::StrPtr:
                            opcode = Opcode::StoreStrPtr;
                            break;

                        case ValueType::Float:
                            opcode = Opcode::StoreFloat;
                            break;
                        case ValueType::Bool:
                            opcode = Opcode::StoreBool;
                            break;
                        default:
                            throw BytecodeError("Invalid type");
                    }

                    writer.write_u8(static_cast<uint8_t>(opcode));
                    writer.write_u16(offset);
                },
                [this](LetDeclaration& let_declaration) {
                    ValueType type = compile_root(
                        *let_declaration.value,
                        let_declaration.cast_type
                    );

                    const Identifier& identifier =
                        std::get<Identifier>(
                            let_declaration.identifier->data
                        );


                    uint16_t offset = 0;

                    if (scopes.back().contains(identifier.name)) {
                        throw BytecodeError("Identifier already assigned");
                    }

                    for (auto it = scopes.begin(); it != scopes.end(); ++it) {
                        offset += it->size();
                    }

                    scopes.back().insert({
                        identifier.name,
                        Symbol{
                            offset,
                            1,
                            identifier.type
                        }
                    });

                    Opcode opcode;

                    switch (type) {
                        case ValueType::Int:
                            opcode = Opcode::StoreInt;
                            break;

                        case ValueType::StrPtr:
                            opcode = Opcode::StoreStrPtr;
                            break;

                        case ValueType::Float:
                            opcode = Opcode::StoreFloat;
                            break;
                        case ValueType::Bool:
                            opcode = Opcode::StoreBool;
                            break;
                        default:
                            throw BytecodeError("Invalid type");
                    }

                    writer.write_u8(static_cast<uint8_t>(opcode));
                    writer.write_u16(offset);
                },
                [this, &loop_context](IfDeclaration& if_declaration) {
                    ValueType type = compile_root(
                        *if_declaration.value,
                        if_declaration.cast_type
                    );
                    if (type != ValueType::Bool) {
                        throw BytecodeError("Non boolean if conditional provided");
                    }
                    writer.write_u8(static_cast<uint8_t>(Opcode::JmpIfFalse));
                    const size_t jump_if_false_patch_pos = writer.code_size();
                    writer.write_u16(0);

                    const size_t program_start_pos = writer.code_size();
                    compile_program(loop_context, *if_declaration.program);
                    writer.patch_u16(jump_if_false_patch_pos, writer.code_size() - program_start_pos + (3*sizeof(uint8_t)));
                    writer.write_u8(static_cast<uint8_t>(Opcode::Jmp));
                    const size_t else_jump_path_pos = writer.code_size();
                    writer.write_u16(0);
                    size_t else_program_start_pos = writer.code_size();
                    compile_program(loop_context, *if_declaration.else_program);
                    writer.patch_u16(else_jump_path_pos, writer.code_size() - else_program_start_pos);
                },
                [this](PrintStatement& print_statement) {
                    ValueType type = compile_root(
                        *print_statement.value,
                        print_statement.cast_type
                    );

                    Opcode opcode;

                    switch (type) {
                        case ValueType::Int:
                            opcode = Opcode::PrintInt;
                            break;

                        case ValueType::StrPtr:
                            opcode = Opcode::PrintStrPtr;
                            break;

                        case ValueType::Float:
                            opcode = Opcode::PrintFloat;
                            break;
                        case ValueType::Bool:
                            opcode = Opcode::PrintBool;
                            break;
                        default:
                            throw BytecodeError("Invalid type");
                    }

                    writer.write_u8(static_cast<uint8_t>(opcode));
        },
        [this](SleepStatement& sleep_statement) {
                    ValueType type = compile_root(
                        *sleep_statement.value,
                        sleep_statement.cast_type
                    );

                    Opcode opcode;

                    switch (type) {
                        case ValueType::Int:
                            opcode = Opcode::SleepInt;
                            break;
                        case ValueType::Float:
                            opcode = Opcode::SleepFloat;
                            break;
                        default:
                            throw BytecodeError("Invalid type");
                    }

                    writer.write_u8(static_cast<uint8_t>(opcode));
                },
                [this, &loop_context](WhileDeclaration& while_declaration) {
                    size_t conditional_start_pos = writer.code_size();
                    loop_context.push_back({
                        conditional_start_pos,
                        scopes.size(),
                        {}
                    });

                    ValueType type = compile_root(
                        *while_declaration.value,
                        while_declaration.cast_type
                    );

                    if (type != ValueType::Bool) {
                        throw BytecodeError("Non boolean while conditional provided");
                    }

                    writer.write_u8(static_cast<uint8_t>(Opcode::JmpIfFalse));
                    const size_t jump_if_false_patch_pos = writer.code_size();
                    writer.write_u16(0);
                    size_t program_start_pos = writer.code_size();
                    compile_program(loop_context, *while_declaration.program);
                    writer.patch_u16(jump_if_false_patch_pos, writer.code_size() - program_start_pos + (3*sizeof(uint8_t)));

                    writer.write_u8(static_cast<uint8_t>(Opcode::JmpBack));
                    writer.write_u16(writer.code_size() - conditional_start_pos + sizeof(uint16_t));
                    for (BreakPatch patch : loop_context.back().break_patches) {
                        size_t current_absolute = writer.code_size();
                        size_t distance = current_absolute - patch.position - sizeof(uint16_t);
                        writer.patch_u16(patch.position, static_cast<uint16_t>(distance));
                    }
                    loop_context.pop_back();
                },
                [this, &loop_context](ContinueStatement&) {
                    if (loop_context.empty()) {
                        throw BytecodeError("break used outside of loop");
                    }
                    clean_scope(loop_context.back().loop_scope);
                    scopes.emplace_back();
                    writer.write_u8(static_cast<uint8_t>(Opcode::JmpBack));
                    writer.write_u16(writer.code_size() + sizeof(uint16_t) - loop_context.back().condition_offset);
                },
                [this, &loop_context](BreakStatement&) {
                    if (loop_context.empty()) {
                        throw BytecodeError("break used outside of loop");
                    }
                    clean_scope(loop_context.back().loop_scope);
                    writer.write_u8(static_cast<uint8_t>(Opcode::Jmp));
                    size_t patch_pos = writer.code_size();
                    writer.write_u16(0);
                    loop_context.back().break_patches.push_back({patch_pos});
                },
                [](auto&) {
                    throw BytecodeError("Unexpected AST node");
                }
            ),
            node->data
        );
    }

    clean_scope(scopes.size() - 1);
    scopes.pop_back();
}

BytecodeWriter Compiler::to_bytecode() {
    std::vector<LoopContext> loop_context;
    compile_program(loop_context, program);

    writer.finalize_header();
    return writer;
}
