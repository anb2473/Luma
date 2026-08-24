#pragma once

#include "../bytecode.h"
#include "codes.h"

class BytecodePrinter {
private:
    std::ostream& out;
    const BytecodeWriter& writer;

    const char* opcode_name(Opcode opcode) const;

    void print_header(const BytecodeWriter& writer) const;
    void print_constants(const BytecodeWriter& writer) const;
    void print_code(const BytecodeWriter& writer) const;

    void print_constant(
        const std::vector<uint8_t>& front,
        size_t& i
    ) const;

    void print_instruction(
        const std::vector<uint8_t>& code,
        size_t& i,
        size_t code_offset
    ) const;

    void print_row(
        size_t offset,
        const std::vector<uint8_t>& buffer,
        size_t byte_start,
        size_t byte_count,
        const char* field,
        const std::string& value
    ) const;

public:
    explicit BytecodePrinter(std::ostream& out, const BytecodeWriter& writer)
        : out(out), writer(writer) {}

    void print() const;
};
