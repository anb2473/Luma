#pragma once

#include "compile/ast_node.h"
#include <cstdint>
#include <cstring>
#include <iostream>
#include <string>
#include <unordered_map>
#include <variant>
#include <vector>
#include "compile/error.h"
#include "compile/value_type.h"

class BytecodeWriter {
private:
    bool headerless;
    std::vector<uint8_t> front;
    std::vector<uint8_t> code;

    static void append_u8(std::vector<uint8_t>& buffer, uint8_t value) {
        buffer.push_back(value);
    }

    static void append_u32(std::vector<uint8_t>& buffer, uint32_t value) {
        const uint8_t* bytes = reinterpret_cast<const uint8_t*>(&value);

        buffer.insert(buffer.end(), bytes, bytes + sizeof(value));
    }

    static void append_f64(std::vector<uint8_t>& buffer, double value) {
        const uint8_t* bytes =
            reinterpret_cast<const uint8_t*>(&value);

        buffer.insert(buffer.end(), bytes, bytes + sizeof(value));
    }

    static void append_u16(std::vector<uint8_t>& buffer, uint16_t value) {
        const uint8_t* bytes =
            reinterpret_cast<const uint8_t*>(&value);

        buffer.insert(buffer.end(), bytes, bytes + sizeof(value));
    }

    static void append_string(
        std::vector<uint8_t>& buffer,
        const std::string& value
    ) {
        for (size_t i = 0; i < value.size(); ++i) {
            char c = value[i];

            if (c != '\\') {
                buffer.push_back(
                    static_cast<uint8_t>(c)
                );
                continue;
            }

            if (++i >= value.size()) {
                throw BytecodeError(
                    "Invalid escape sequence"
                );
            }

            char escaped = value[i];

            switch (escaped) {
                case 'n':
                    buffer.push_back('\n');
                    break;

                case 'r':
                    buffer.push_back('\r');
                    break;

                case 't':
                    buffer.push_back('\t');
                    break;

                case '\\':
                    buffer.push_back('\\');
                    break;

                case '"':
                    buffer.push_back('"');
                    break;

                case '\'':
                    buffer.push_back('\'');
                    break;

                case '0':
                    buffer.push_back('\0');
                    break;

                default:
                    throw BytecodeError(
                        "Invalid escape sequence"
                    );
            }
        }
    }

    void append_code(std::vector<uint8_t>& buffer, const std::vector<uint8_t>& code_data) {
        buffer.insert(buffer.end(), code_data.begin(), code_data.end());
    }

    void append_front(std::vector<uint8_t>& buffer, const std::vector<uint8_t>& front_data) {
        buffer.insert(buffer.end(), front_data.begin(), front_data.end());
    }

public:
    BytecodeWriter(bool headerless)
        : headerless(headerless), front(headerless ? 0 : 4) {}

    void finalize_header() {
        if (headerless) {
            return;
        }

        const uint32_t code_offset =
            static_cast<uint32_t>(front.size());

        std::memcpy(
            front.data(),
            &code_offset,
            sizeof(code_offset)
        );
    }

    void write_u8(uint8_t value) {
        append_u8(code, value);
    }

    void write_u32(uint32_t value) {
        append_u32(code, value);
    }

    void write_f64(double value) {
        append_f64(code, value);
    }

    void write_u16(uint16_t value) {
        append_u16(code, value);
    }

    void write_string(const std::string& value) {
        append_string(code, value);
    }

    void write_front_u8(uint8_t value) {
        append_u8(front, value);
    }

    void write_front_u32(uint32_t value) {
        append_u32(front, value);
    }

    void write_front_f64(double value) {
        append_f64(front, value);
    }

    void write_front_u16(uint16_t value) {
        append_u16(front, value);
    }

    void write_front_string(
        const std::string& value
    ) {
        size_t size = 0;

        for (size_t i = 0; i < value.size(); ++i) {
            if (value[i] == '\\') {
                ++i;
            }

            ++size;
        }

        if (size > UINT16_MAX) {
            throw BytecodeError(
                "String literal exceeds maximum size"
            );
        }

        append_u16(
            front,
            static_cast<uint16_t>(size)
        );

        append_string(
            front,
            value
        );
    }

    void flush_file(std::ostream& out) {
        out.write(
            reinterpret_cast<const char*>(front.data()),
            static_cast<std::streamsize>(front.size())
        );

        out.write(
            reinterpret_cast<const char*>(code.data()),
            static_cast<std::streamsize>(code.size())
        );
    }

    size_t code_size() const {
        return code.size();
    }

    size_t front_size() const {
        return front.size();
    }

    const std::vector<uint8_t>& front_data() const {
        return front;
    }

    const std::vector<uint8_t>& code_data() const {
        return code;
    }

    void write_front(const std::vector<uint8_t>& front_data) {
        append_front(front, front_data);
    }

    void write_code(const std::vector<uint8_t>& code_data) {
        append_code(code, code_data);
    }

    void patch_u16(size_t position, uint16_t value) {
        if (position + sizeof(uint16_t) > code.size()) {
            throw BytecodeError("Invalid patch position");
        }

        std::memcpy(code.data() + position, &value, sizeof(value));
    }
};

struct Symbol {
    uint16_t offset;
    uint16_t size;
};

using SymbolTable = std::unordered_map<std::string, Symbol>;

enum ConstantType {
    String
};

using Value = std::variant<
    std::string
>;

struct Constant {
    ConstantType type;
    Value value;
    uint32_t pos;
};

struct BreakPatch {
    size_t position;
};

struct LoopContext {
    size_t condition_offset;
    std::vector<BreakPatch> break_patches;
};

class Compiler {
    private:
        Program& program;
        bool headerless;
        BytecodeWriter writer;
        std::vector<SymbolTable> scopes;
        ValueType compile_root(const ASTNode &root, ValueType cast_type);

        void compile_program(std::vector<LoopContext> &loop_context, Program &program);

        std::unordered_map<Value, Constant> constants;
        ValueType cast_str_to(ValueType superior_type, const StringLiteral& string_literal);
        ValueType cast_float_to(ValueType superior_type, const FloatLiteral& float_literal);
        ValueType cast_int_to(ValueType superior_type, const IntLiteral& int_literal);
        ValueType cast_identifier_to(ValueType superior_type, const Identifier& identifier, ValueType identifier_type);
        ValueType cast(ValueType superior_type, const ASTNode& node);
        ValueType cast_bool_to(ValueType superior_type, const BoolLiteral& bool_literal);
    public:
        Compiler(
            Program& program,
            const std::vector<SymbolTable>& parent_scopes,
            bool headerless = false
        )
            : program(program),
            headerless(headerless),
            writer(headerless),
            scopes(parent_scopes)
        {
            scopes.push_back({});
        }
        BytecodeWriter to_bytecode();
};
