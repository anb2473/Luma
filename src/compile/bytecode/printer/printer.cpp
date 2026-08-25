#include "printer.h"

#include <cstdint>
#include <cstring>
#include <iomanip>
#include <iostream>
#include <stdexcept>
#include <string>
#include <vector>

namespace {

constexpr size_t OFFSET_WIDTH = 8;
constexpr size_t BYTES_WIDTH = 28;
constexpr size_t FIELD_WIDTH = 20;

uint16_t read_u16(
    const std::vector<uint8_t>& buffer,
    size_t offset
) {
    if (offset > buffer.size() ||
        sizeof(uint16_t) > buffer.size() - offset) {
        throw std::runtime_error(
            "Bytecode printer attempted to read uint16_t past buffer"
        );
    }

    uint16_t value;

    std::memcpy(
        &value,
        buffer.data() + offset,
        sizeof(value)
    );

    return value;
}

uint32_t read_u32(
    const std::vector<uint8_t>& buffer,
    size_t offset
) {
    if (offset > buffer.size() ||
        sizeof(uint32_t) > buffer.size() - offset) {
        throw std::runtime_error(
            "Bytecode printer attempted to read uint32_t past buffer"
        );
    }

    uint32_t value;

    std::memcpy(
        &value,
        buffer.data() + offset,
        sizeof(value)
    );

    return value;
}

int32_t read_i32(
    const std::vector<uint8_t>& buffer,
    size_t offset
) {
    if (offset > buffer.size() ||
        sizeof(int32_t) > buffer.size() - offset) {
        throw std::runtime_error(
            "Bytecode printer attempted to read int32_t past buffer"
        );
    }

    int32_t value;

    std::memcpy(
        &value,
        buffer.data() + offset,
        sizeof(value)
    );

    return value;
}

double read_f64(
    const std::vector<uint8_t>& buffer,
    size_t offset
) {
    if (offset > buffer.size() ||
        sizeof(double) > buffer.size() - offset) {
        throw std::runtime_error(
            "Bytecode printer attempted to read double past buffer"
        );
    }

    double value;

    std::memcpy(
        &value,
        buffer.data() + offset,
        sizeof(value)
    );

    return value;
}

bool is_known_opcode(uint8_t value) {
    switch (static_cast<Opcode>(value)) {
        case Opcode::PushInt:
        case Opcode::PushFloat:
        case Opcode::PushConst:
        case Opcode::PushBool:

        case Opcode::AddInt:
        case Opcode::SubInt:
        case Opcode::MultInt:
        case Opcode::DivInt:

        case Opcode::AddFloat:
        case Opcode::SubFloat:
        case Opcode::MultFloat:
        case Opcode::DivFloat:

        case Opcode::SubUnaryInt:
        case Opcode::SubUnaryFloat:

        case Opcode::AddStrPtr:

        case Opcode::StoreInt:
        case Opcode::StoreFloat:
        case Opcode::StoreConst:
        case Opcode::StoreStrPtr:
        case Opcode::StoreBool:

        case Opcode::LoadInt:
        case Opcode::LoadFloat:
        case Opcode::LoadStrPtr:
        case Opcode::LoadBool:

        case Opcode::PrintInt:
        case Opcode::PrintFloat:
        case Opcode::PrintStrPtr:
        case Opcode::PrintBool:

        case Opcode::FloatToStrPtr:
        case Opcode::IntToFloat:
        case Opcode::IntToStrPtr:

        case Opcode::IsEqInt:
        case Opcode::IsEqFloat:
        case Opcode::IsEqBool:

        case Opcode::IsNotEqInt:
        case Opcode::IsNotEqFloat:
        case Opcode::IsNotEqBool:

        case Opcode::IsGreaterFloat:
        case Opcode::IsGreaterInt:

        case Opcode::IsGreaterOrEqFloat:
        case Opcode::IsGreaterOrEqInt:

        case Opcode::IsLessOrEqFloat:
        case Opcode::IsLessOrEqInt:

        case Opcode::IsLessFloat:
        case Opcode::IsLessInt:

        case Opcode::JmpIfFalse:
        case Opcode::Jmp:
        case Opcode::JmpBack:

        case Opcode::PopStack:
        case Opcode::DelStrPtr:

        case Opcode::SleepInt:
        case Opcode::SleepFloat:

            return true;

        default:
            return false;
    }
}

} // namespace


const char* BytecodePrinter::opcode_name(Opcode opcode) const {
    switch (opcode) {
        case Opcode::PushInt:
            return "PUSH_INT";

        case Opcode::PushFloat:
            return "PUSH_FLOAT";

        case Opcode::PushConst:
            return "PUSH_CONST";

        case Opcode::PushBool:
            return "PUSH_BOOL";

        case Opcode::AddInt:
            return "ADD_INT";

        case Opcode::SubInt:
            return "SUB_INT";

        case Opcode::MultInt:
            return "MULT_INT";

        case Opcode::DivInt:
            return "DIV_INT";

        case Opcode::AddFloat:
            return "ADD_FLOAT";

        case Opcode::SubFloat:
            return "SUB_FLOAT";

        case Opcode::MultFloat:
            return "MULT_FLOAT";

        case Opcode::DivFloat:
            return "DIV_FLOAT";

        case Opcode::SubUnaryInt:
            return "SUB_UNARY_INT";

        case Opcode::SubUnaryFloat:
            return "SUB_UNARY_FLOAT";

        case Opcode::AddStrPtr:
            return "ADD_STR_PTR";

        case Opcode::StoreInt:
            return "STORE_INT";

        case Opcode::StoreFloat:
            return "STORE_FLOAT";

        case Opcode::StoreConst:
            return "STORE_CONST";

        case Opcode::StoreStrPtr:
            return "STORE_STR_PTR";

        case Opcode::StoreBool:
            return "STORE_BOOL";

        case Opcode::LoadInt:
            return "LOAD_INT";

        case Opcode::LoadFloat:
            return "LOAD_FLOAT";

        case Opcode::LoadStrPtr:
            return "LOAD_STR_PTR";

        case Opcode::LoadBool:
            return "LOAD_BOOL";

        case Opcode::PrintInt:
            return "PRINT_INT";

        case Opcode::PrintFloat:
            return "PRINT_FLOAT";

        case Opcode::PrintStrPtr:
            return "PRINT_STR_PTR";

        case Opcode::PrintBool:
            return "PRINT_BOOL";

        case Opcode::FloatToStrPtr:
            return "FLOAT_TO_STR_PTR";

        case Opcode::IntToFloat:
            return "INT_TO_FLOAT";

        case Opcode::IntToStrPtr:
            return "INT_TO_STR_PTR";

        case Opcode::IsEqInt:
            return "IS_EQ_INT";

        case Opcode::IsEqFloat:
            return "IS_EQ_FLOAT";

        case Opcode::IsEqBool:
            return "IS_EQ_BOOL";

        case Opcode::IsNotEqInt:
            return "IS_NOT_EQ_INT";

        case Opcode::IsNotEqFloat:
            return "IS_NOT_EQ_FLOAT";

        case Opcode::IsNotEqBool:
            return "IS_NOT_EQ_BOOL";

        case Opcode::IsGreaterFloat:
            return "IS_GREATER_FLOAT";

        case Opcode::IsGreaterInt:
            return "IS_GREATER_INT";

        case Opcode::IsGreaterOrEqFloat:
            return "IS_GREATER_OR_EQ_FLOAT";

        case Opcode::IsGreaterOrEqInt:
            return "IS_GREATER_OR_EQ_INT";

        case Opcode::IsLessOrEqFloat:
            return "IS_LESS_OR_EQ_FLOAT";

        case Opcode::IsLessOrEqInt:
            return "IS_LESS_OR_EQ_INT";

        case Opcode::IsLessFloat:
            return "IS_LESS_FLOAT";

        case Opcode::IsLessInt:
            return "IS_LESS_INT";

        case Opcode::JmpIfFalse:
            return "JMP_IF_FALSE";

        case Opcode::Jmp:
            return "JMP";

        case Opcode::JmpBack:
            return "JMP_BACK";

        case Opcode::PopStack:
            return "POP_STACK";

        case Opcode::DelStrPtr:
            return "DEL_STR_PTR";

        case Opcode::SleepInt:
            return "SLEEP_INT";

        case Opcode::SleepFloat:
            return "SLEEP_FLOAT";

        default:
            return "UNKNOWN";
    }
}


void BytecodePrinter::print_row(
    size_t offset,
    const std::vector<uint8_t>& buffer,
    size_t byte_start,
    size_t byte_count,
    const char* field,
    const std::string& value
) const {
    if (byte_start > buffer.size() ||
        byte_count > buffer.size() - byte_start) {
        throw std::runtime_error(
            "Bytecode printer attempted to read past buffer"
        );
    }

    out
        << "    "
        << std::right
        << std::setw(OFFSET_WIDTH)
        << std::setfill('0')
        << offset
        << std::setfill(' ')
        << "    ";

    for (size_t i = 0; i < byte_count; ++i) {
        if (i != 0) {
            out << ' ';
        }

        out
            << std::hex
            << std::setw(2)
            << std::setfill('0')
            << static_cast<unsigned int>(
                buffer[byte_start + i]
            )
            << std::dec
            << std::setfill(' ');
    }

    const size_t byte_text_width =
        byte_count == 0
            ? 0
            : byte_count * 2 + byte_count - 1;

    if (byte_text_width < BYTES_WIDTH) {
        out << std::string(
            BYTES_WIDTH - byte_text_width,
            ' '
        );
    }

    out
        << std::left
        << std::setw(FIELD_WIDTH)
        << field
        << std::right
        << value
        << '\n';
}


void BytecodePrinter::print_header(
    const BytecodeWriter& writer
) const {
    const auto& front = writer.front_data();

    if (front.size() < sizeof(uint32_t)) {
        throw std::runtime_error(
            "Truncated bytecode header"
        );
    }

    const uint32_t code_offset = read_u32(front, 0);

    if (code_offset != front.size()) {
        throw std::runtime_error(
            "Invalid code offset in bytecode header"
        );
    }

    out << "\nHeader:\n";

    out
        << "    "
        << std::left
        << std::setw(OFFSET_WIDTH)
        << "OFFSET"
        << "    "
        << std::setw(BYTES_WIDTH)
        << "BYTES"
        << std::setw(FIELD_WIDTH)
        << "FIELD"
        << "VALUE"
        << '\n';

    print_row(
        0,
        front,
        0,
        sizeof(uint32_t),
        "CODE_OFFSET",
        std::to_string(code_offset)
    );
}


void BytecodePrinter::print_constant(
    const std::vector<uint8_t>& front,
    size_t& i
) const {
    const size_t length_offset = i;

    const uint16_t length =
        read_u16(front, i);

    print_row(
        length_offset,
        front,
        length_offset,
        sizeof(uint16_t),
        "LENGTH",
        std::to_string(length)
    );

    i += sizeof(uint16_t);

    if (static_cast<size_t>(length) >
        front.size() - i) {
        throw std::runtime_error(
            "Truncated string constant: missing value"
        );
    }

    const size_t value_offset = i;

    const std::string value(
        reinterpret_cast<const char*>(
            front.data() + i
        ),
        length
    );

    print_row(
        value_offset,
        front,
        value_offset,
        length,
        "VALUE",
        "\"" + value + "\""
    );

    i += length;
}


void BytecodePrinter::print_constants(
    const BytecodeWriter& writer
) const {
    const auto& front = writer.front_data();

    if (front.size() < sizeof(uint32_t)) {
        throw std::runtime_error(
            "Truncated bytecode header"
        );
    }

    const uint32_t code_offset =
        read_u32(front, 0);

    if (code_offset != front.size()) {
        throw std::runtime_error(
            "Invalid code offset in bytecode header"
        );
    }

    out << "\nConstants:\n";

    out
        << "    "
        << std::left
        << std::setw(OFFSET_WIDTH)
        << "OFFSET"
        << "    "
        << std::setw(BYTES_WIDTH)
        << "BYTES"
        << std::setw(FIELD_WIDTH)
        << "FIELD"
        << "VALUE"
        << '\n';

    size_t i = sizeof(uint32_t);

    while (i < front.size()) {
        print_constant(front, i);
    }
}


void BytecodePrinter::print_instruction(
    const std::vector<uint8_t>& code,
    size_t& i,
    size_t code_offset
) const {
    if (i >= code.size()) {
        return;
    }

    const size_t opcode_offset = i;
    const uint8_t raw_opcode = code[i];

    if (!is_known_opcode(raw_opcode)) {
        throw std::runtime_error(
            "Unknown opcode " +
            std::to_string(
                static_cast<unsigned int>(raw_opcode)
            ) +
            " at code offset " +
            std::to_string(
                code_offset + opcode_offset
            )
        );
    }

    const Opcode opcode =
        static_cast<Opcode>(raw_opcode);

    print_row(
        code_offset + opcode_offset,
        code,
        opcode_offset,
        sizeof(uint8_t),
        "OPCODE",
        opcode_name(opcode)
    );

    ++i;

    // ------------------------------------------------------------
    // uint32 / int32 operands
    // ------------------------------------------------------------

    switch (opcode) {
        case Opcode::PushInt: {
            const int32_t value =
                read_i32(code, i);

            print_row(
                code_offset + i,
                code,
                i,
                sizeof(int32_t),
                "OPERAND",
                std::to_string(value)
            );

            i += sizeof(int32_t);
            return;
        }

        case Opcode::PushConst: {
            const uint32_t offset =
                read_u32(code, i);

            print_row(
                code_offset + i,
                code,
                i,
                sizeof(uint32_t),
                "CONST_OFFSET",
                std::to_string(offset)
            );

            i += sizeof(uint32_t);
            return;
        }

        default:
            break;
    }

    // ------------------------------------------------------------
    // double operand
    // ------------------------------------------------------------

    if (opcode == Opcode::PushFloat) {
        const double value =
            read_f64(code, i);

        print_row(
            code_offset + i,
            code,
            i,
            sizeof(double),
            "OPERAND",
            std::to_string(value)
        );

        i += sizeof(double);
        return;
    }

    // ------------------------------------------------------------
    // uint8 operand
    // ------------------------------------------------------------

    if (opcode == Opcode::PushBool) {
        if (i >= code.size()) {
            throw std::runtime_error(
                "Truncated PUSH_BOOL"
            );
        }

        const uint8_t value = code[i];

        if (value > 1) {
            throw std::runtime_error(
                "Invalid boolean operand"
            );
        }

        print_row(
            code_offset + i,
            code,
            i,
            sizeof(uint8_t),
            "OPERAND",
            value ? "true" : "false"
        );

        ++i;
        return;
    }

    // ------------------------------------------------------------
    // uint16 stack offset
    // ------------------------------------------------------------

    switch (opcode) {
        case Opcode::StoreInt:
        case Opcode::StoreFloat:
        case Opcode::StoreConst:
        case Opcode::StoreStrPtr:
        case Opcode::StoreBool:

        case Opcode::LoadInt:
        case Opcode::LoadFloat:
        case Opcode::LoadStrPtr:
        case Opcode::LoadBool: {
            const uint16_t offset =
                read_u16(code, i);

            print_row(
                code_offset + i,
                code,
                i,
                sizeof(uint16_t),
                "STACK_OFFSET",
                std::to_string(offset)
            );

            i += sizeof(uint16_t);
            return;
        }

        default:
            break;
    }

    // ------------------------------------------------------------
    // uint16 jump operand
    // ------------------------------------------------------------

    switch (opcode) {
        case Opcode::JmpIfFalse:
        case Opcode::Jmp:
        case Opcode::JmpBack: {
            const uint16_t offset =
                read_u16(code, i);

            print_row(
                code_offset + i,
                code,
                i,
                sizeof(uint16_t),
                "JUMP_OFFSET",
                std::to_string(offset)
            );

            i += sizeof(uint16_t);
            return;
        }

        default:
            break;
    }

    // ------------------------------------------------------------
    // No-operand instructions
    // ------------------------------------------------------------

    switch (opcode) {
        case Opcode::AddInt:
        case Opcode::SubInt:
        case Opcode::MultInt:
        case Opcode::DivInt:

        case Opcode::AddFloat:
        case Opcode::SubFloat:
        case Opcode::MultFloat:
        case Opcode::DivFloat:

        case Opcode::SubUnaryInt:
        case Opcode::SubUnaryFloat:

        case Opcode::AddStrPtr:

        case Opcode::PrintInt:
        case Opcode::PrintFloat:
        case Opcode::PrintStrPtr:
        case Opcode::PrintBool:

        case Opcode::FloatToStrPtr:
        case Opcode::IntToFloat:
        case Opcode::IntToStrPtr:

        case Opcode::IsEqInt:
        case Opcode::IsEqFloat:
        case Opcode::IsEqBool:

        case Opcode::IsNotEqInt:
        case Opcode::IsNotEqFloat:
        case Opcode::IsNotEqBool:

        case Opcode::IsGreaterFloat:
        case Opcode::IsGreaterInt:

        case Opcode::IsGreaterOrEqFloat:
        case Opcode::IsGreaterOrEqInt:

        case Opcode::IsLessOrEqFloat:
        case Opcode::IsLessOrEqInt:

        case Opcode::IsLessFloat:
        case Opcode::IsLessInt:

        case Opcode::SleepInt:
        case Opcode::SleepFloat:

        case Opcode::PopStack:
        case Opcode::DelStrPtr:

            return;

        default:
            break;
    }

    throw std::runtime_error(
        "Printer has no operand definition for opcode " +
        std::to_string(
            static_cast<unsigned int>(raw_opcode)
        )
    );
}


void BytecodePrinter::print_code(
    const BytecodeWriter& writer
) const {
    const auto& code = writer.code_data();
    const auto& front = writer.front_data();

    if (front.size() < sizeof(uint32_t)) {
        throw std::runtime_error(
            "Truncated bytecode header"
        );
    }

    const uint32_t code_offset =
        read_u32(front, 0);

    if (code_offset != front.size()) {
        throw std::runtime_error(
            "Invalid code offset"
        );
    }

    out << "\nCode:\n";

    out
        << "    "
        << std::left
        << std::setw(OFFSET_WIDTH)
        << "OFFSET"
        << "    "
        << std::setw(BYTES_WIDTH)
        << "BYTES"
        << std::setw(FIELD_WIDTH)
        << "FIELD"
        << "VALUE"
        << '\n';

    size_t i = 0;

    while (i < code.size()) {
        print_instruction(
            code,
            i,
            code_offset
        );
    }
}


void BytecodePrinter::print() const {
    print_header(writer);
    print_constants(writer);
    print_code(writer);
}
