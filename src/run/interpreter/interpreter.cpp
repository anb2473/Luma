#include "interpreter.h"
#include "codes.h"
#include "../error.h"
#include <iostream>
#include <string>
#include <cstring>
#include <cstdint>

Header Interpreter::read_header() {
    if (source.size() < 4) {
        throw InterpreterError("Missing header");
    }

    uint32_t value;

    std::memcpy(
        &value,
        source.data(),
        sizeof(value)
    );

    return Header{
        value
    };
}

void Stack::push_u64(uint64_t value) {
    if (stack_pointer <= stack_base) {
        throw InterpreterError("Stack overflow");
    }

    stack_pointer--;

    *stack_pointer = value;
}

void Stack::set_u64(uint64_t value, uint16_t offset) {
    if (offset >= stack_size) {
        throw InterpreterError("Stack overflow");
    }

    uint64_t* ptr = stack_end - 1 - offset;

    *ptr = value;

    if (offset >= size()) {
        stack_pointer = stack_end - (offset + 1);;
    }
}

void Stack::push_f64(double value) {
    if (stack_pointer <= stack_base) {
        throw InterpreterError("Stack overflow");
    }

    stack_pointer--;

    std::memcpy(
        stack_pointer,
        &value,
        sizeof(value)
    );
}

void Stack::set_f64(double value, uint16_t offset) {
    if (offset >= stack_size) {
        throw InterpreterError("Stack overflow");
    }

    uint64_t* ptr = stack_end - 1 - offset;

    std::memcpy(
        ptr,
        &value,
        sizeof(value)
    );

    if (offset >= size()) {
        stack_pointer = stack_end - (offset + 1);;
    }
}

uint64_t Stack::pop_u64() {
    if (stack_pointer >= stack_end) {
        throw InterpreterError("Stack underflow");
    }

    uint64_t value;

    value = *stack_pointer;

    stack_pointer++;

    return value;
}

double Stack::pop_f64() {
    if (stack_pointer >= stack_end) {
        throw InterpreterError("Stack underflow");
    }

    double value;
    std::memcpy(
        &value,
        stack_pointer,
        sizeof(value)
    );

    stack_pointer++;

    return value;
}

uint64_t Stack::stack_at(size_t offset) const {
    if (offset >= stack_size) {
        throw InterpreterError("Stack index out of bounds");
    }

    return stack_end[-1 - offset];
}

void Stack::pop_n(uint16_t count) {
    if (count > size()) {
        throw std::runtime_error("Stack underflow");
    }

    stack_pointer += count;
}

uint8_t Interpreter::read_byte() {
    if (pos > source.size() - sizeof(uint8_t)) {
        throw InterpreterError("Truncated byte");
    }

    return source[pos++];
}

uint32_t Interpreter::read_u32() {
    if (pos > source.size() - sizeof(uint32_t)) {
        throw InterpreterError("Truncated u32");
    }

    uint32_t val;

    std::memcpy(
        &val,
        source.data() + pos,
        sizeof(val)
    );
    pos += sizeof(val);
    return val;
}

uint16_t Interpreter::read_u16() {
    if (pos > source.size() - sizeof(uint16_t)) {
        throw InterpreterError("Truncated u16");
    }

    uint16_t val;

    std::memcpy(
        &val,
        source.data() + pos,
        sizeof(val)
    );
    pos += sizeof(val);
    return val;
}

double Interpreter::read_f64() {
    if (pos > source.size() - sizeof(double)) {
        throw InterpreterError("Truncated f64");
    }

    double val;

    std::memcpy(
        &val,
        source.data() + pos,
        sizeof(val)
    );
    pos += sizeof(val);
    return val;
}

std::string Interpreter::read_const_str(uint32_t offset) {

    if (offset + sizeof(uint16_t) > source.size()) {
        throw InterpreterError("Constant offset out of range");
    }

    uint16_t size;
    std::memcpy(
        &size,
        source.data() + offset,
        sizeof(size)
    );

    if (offset + sizeof(size) + size > source.size()) {
        throw InterpreterError("Constant offset out of range");
    }

    std::string val(size, '\0');
    std::memcpy(
        val.data(),
        source.data() + offset + sizeof(size),
        size
    );
    return val;
}

void Interpreter::jump_forwards(size_t dist) {
    if (dist > source.size() - pos) {
        throw InterpreterError("Code offset out of range");
    }

    pos += dist;
}

void Interpreter::jump_backwards(size_t dist) {
    if (dist > pos) {
        throw InterpreterError("Code offset out of range");
    }

    pos -= dist;
}

void Interpreter::store_int() {
    uint16_t offset = read_u16();
    uint32_t value = static_cast<uint32_t>(opstack.pop_u64());
    stack.set_u64(
        value,
        offset
    );
}

bool is_const_ptr(uint64_t value) {
    return value & (1ULL << 63);
}

void Interpreter::store_ptr() {
    uint16_t offset = read_u16();
    uint64_t value = opstack.pop_u64();
    stack.set_u64(
        value,
        offset
    );
}

void Interpreter::store_float() {
    uint16_t offset = read_u16();
    double value = opstack.pop_f64();
    stack.set_f64(
        value,
        offset
    );
}

void Interpreter::store_bool() {
    uint16_t offset = read_u16();
    uint64_t value = opstack.pop_u64();
    stack.set_u64(
        value,
        offset
    );
}

void Interpreter::push_int() {
    const uint32_t operand = read_u32();
    opstack.push_u64(operand);
}

void Interpreter::push_bool() {
    const bool operand = static_cast<bool>(read_byte());
    opstack.push_u64(operand);
}

void Interpreter::push_float() {
    opstack.push_f64(read_f64());
}

void Interpreter::push_const() {
    const uint32_t pos = read_u32();
    uint64_t tagged = static_cast<uint64_t>(pos) | (1ULL << 63);
    opstack.push_u64(tagged);
}

void Interpreter::add_int() {
    const uint32_t a = static_cast<uint32_t>(opstack.pop_u64());
    const uint32_t b = static_cast<uint32_t>(opstack.pop_u64());
    opstack.push_u64(
        a + b
    );
}

void Interpreter::is_eq_int() {
    const uint32_t b = static_cast<uint32_t>(opstack.pop_u64());
    const uint32_t a = static_cast<uint32_t>(opstack.pop_u64());
    opstack.push_u64(
        a == b
    );
}

void Interpreter::is_not_eq_int() {
    const uint32_t b = static_cast<uint32_t>(opstack.pop_u64());
    const uint32_t a = static_cast<uint32_t>(opstack.pop_u64());
    opstack.push_u64(
        a != b
    );
}

void Interpreter::is_greater_or_eq_int() {
    const uint32_t b = static_cast<uint32_t>(opstack.pop_u64());
    const uint32_t a = static_cast<uint32_t>(opstack.pop_u64());
    opstack.push_u64(
        a >= b
    );
}

void Interpreter::is_less_or_eq_int() {
    const uint32_t b = static_cast<uint32_t>(opstack.pop_u64());
    const uint32_t a = static_cast<uint32_t>(opstack.pop_u64());
    opstack.push_u64(
        a <= b
    );
}

void Interpreter::is_greater_int() {
    const uint32_t b = static_cast<uint32_t>(opstack.pop_u64());
    const uint32_t a = static_cast<uint32_t>(opstack.pop_u64());
    opstack.push_u64(
        a > b
    );
}

void Interpreter::is_less_int() {
    const uint32_t b = static_cast<uint32_t>(opstack.pop_u64());
    const uint32_t a = static_cast<uint32_t>(opstack.pop_u64());
    opstack.push_u64(
        a < b
    );
}

void Interpreter::add_float() {
    const double b = opstack.pop_f64();
    const double a = opstack.pop_f64();
    opstack.push_f64(
        a + b
    );
}

void Interpreter::is_eq_float() {
    const double b = opstack.pop_f64();
    const double a = opstack.pop_f64();
    opstack.push_u64(
        a == b
    );
}

void Interpreter::is_not_eq_float() {
    const double b = opstack.pop_f64();
    const double a = opstack.pop_f64();
    opstack.push_u64(
        a != b
    );
}

void Interpreter::is_less_or_eq_float() {
    const double b = opstack.pop_f64();
    const double a = opstack.pop_f64();
    opstack.push_u64(
        a <= b
    );
}

void Interpreter::is_greater_or_eq_float() {
    const double b = opstack.pop_f64();
    const double a = opstack.pop_f64();
    opstack.push_u64(
        a >= b
    );
}

void Interpreter::is_less_float() {
    const double b = opstack.pop_f64();
    const double a = opstack.pop_f64();
    opstack.push_u64(
        a < b
    );
}

void Interpreter::is_greater_float() {
    const double b = opstack.pop_f64();
    const double a = opstack.pop_f64();
    opstack.push_u64(
        a > b
    );
}

void Interpreter::is_eq_bool() {
    const uint64_t b = opstack.pop_u64();
    const uint64_t a = opstack.pop_u64();
    opstack.push_u64(
        a == b
    );
}

void Interpreter::is_not_eq_bool() {
    const uint64_t b = opstack.pop_u64();
    const uint64_t a = opstack.pop_u64();
    opstack.push_u64(
        a != b
    );
}

void Interpreter::add_str_ptr() {
    uint64_t a_ptr = opstack.pop_u64();
    uint64_t b_ptr = opstack.pop_u64();

    std::string a;
    std::string b;

    if (is_const_ptr(a_ptr)) {
        a = read_const_str(static_cast<uint32_t>(a_ptr));
    } else {
        a = *reinterpret_cast<std::string*>(a_ptr);
    }
    if (is_const_ptr(b_ptr)) {
        b = read_const_str(static_cast<uint32_t>(b_ptr));
    } else {
        b = *reinterpret_cast<std::string*>(b_ptr);
    }

    std::string* result = new std::string(b + a);
    opstack.push_u64(
        reinterpret_cast<uint64_t>(result)
    );
}

void Interpreter::print_int() {
    const uint32_t a = static_cast<uint32_t>(opstack.pop_u64());
    out << a;
}

void Interpreter::print_bool() {
    const bool a = static_cast<bool>(opstack.pop_u64());
    out << a;
}

void Interpreter::print_float() {
    const double a = opstack.pop_f64();
    out << a;
}

void Interpreter::print_str_ptr() {
    uint64_t a_ptr = opstack.pop_u64();

    std::string a;
    if (is_const_ptr(a_ptr)) {
        a = read_const_str(static_cast<uint32_t>(a_ptr));
    } else {
        a = *reinterpret_cast<std::string*>(a_ptr);
    }

    out << a;
}

void Interpreter::sub_int() {
    const uint64_t b = static_cast<uint32_t>(opstack.pop_u64());
    const uint64_t a = static_cast<uint32_t>(opstack.pop_u64());
    opstack.push_u64(
        a - b
    );
}

void Interpreter::sub_float() {
    const double b = opstack.pop_f64();
    const double a = opstack.pop_f64();
    opstack.push_f64(
        a - b
    );
}

void Interpreter::mult_int() {
    const uint64_t a = static_cast<uint32_t>(opstack.pop_u64());
    const uint64_t b = static_cast<uint32_t>(opstack.pop_u64());
    opstack.push_u64(
        a * b
    );
}

void Interpreter::mult_float() {
    const double a = opstack.pop_f64();
    const double b = opstack.pop_f64();
    opstack.push_f64(
        a * b
    );
}

void Interpreter::div_int() {
    const uint64_t b = static_cast<uint32_t>(opstack.pop_u64());
    const uint64_t a = static_cast<uint32_t>(opstack.pop_u64());
    opstack.push_u64(
        a / b
    );
}

void Interpreter::div_float() {
    const double b = opstack.pop_f64();
    const double a = opstack.pop_f64();
    opstack.push_f64(
        a / b
    );
}

void Interpreter::load_int() {
    uint16_t offset = read_u16();
    uint64_t value = (stack.stack_at(offset));
    opstack.push_u64(value);
}

void Interpreter::load_bool() {
    uint16_t offset = read_u16();
    uint64_t value = (stack.stack_at(offset));
    opstack.push_u64(value);
}

void Interpreter::load_float() {
    uint16_t offset = read_u16();
    uint64_t bits = stack.stack_at(offset);
    double value;
    std::memcpy(
        &value,
        &bits,
        sizeof(bits)
    );
    opstack.push_f64(value);
}

void Interpreter::load_str_ptr() {
    uint16_t offset = read_u16();
    uint64_t value = (stack.stack_at(offset));
    opstack.push_u64(value);
}

void Interpreter::int_to_float() {
    uint32_t value = static_cast<uint32_t>(opstack.pop_u64());
    opstack.push_f64(
        static_cast<double>(value)
    );
}

void Interpreter::int_to_str_ptr() {
    uint32_t value = static_cast<uint32_t>(opstack.pop_u64());
    std::string* ptr = new std::string(std::to_string(value));
    opstack.push_u64(
        reinterpret_cast<uint64_t>(ptr)
    );
}

void Interpreter::float_to_str_ptr() {
    double value = opstack.pop_f64();
    std::string* ptr = new std::string(std::to_string(value));
    opstack.push_u64(
        reinterpret_cast<uint64_t>(ptr)
    );
}

void Interpreter::jump_if_false() {
    uint8_t value = static_cast<uint8_t>(opstack.pop_u64());
    const uint32_t dist = read_u16();
    if (!value) {
        jump_forwards(dist);
    }
}

void Interpreter::jump() {
    const uint32_t dist = read_u16();
    jump_forwards(dist);
}

void Interpreter::jump_back() {
    const uint32_t dist = read_u16();
    jump_backwards(dist);
}

void Interpreter::pop_stack() {
    const uint32_t n = read_u16();
    stack.pop_n(n);
}

void Interpreter::run_code() {
    while (pos < source.size()) {
        const Opcode opcode = static_cast<Opcode>(read_byte());
        switch (opcode) {
            case Opcode::AddInt:
                add_int();
                break;
            case Opcode::AddFloat:
                add_float();
                break;
            case Opcode::AddStrPtr:
                add_str_ptr();
                break;
            case Opcode::StoreInt:
                store_int();
                break;
            case Opcode::StoreFloat:
                store_float();
                break;
            case Opcode::StoreStrPtr:
                store_ptr();
                break;
            case Opcode::PushInt:
                push_int();
                break;
            case Opcode::PushFloat:
                push_float();
                break;
            case Opcode::PushConst:
                push_const();
                break;
            case Opcode::SubInt:
                sub_int();
                break;
            case Opcode::SubFloat:
                sub_float();
                break;
            case Opcode::MultInt:
                mult_int();
                break;
            case Opcode::MultFloat:
                mult_float();
                break;
            case Opcode::DivInt:
                div_int();
                break;
            case Opcode::DivFloat:
                div_float();
                break;
            case Opcode::PrintInt:
                print_int();
                break;
            case Opcode::PrintFloat:
                print_float();
                break;
            case Opcode::PrintStrPtr:
                print_str_ptr();
                break;
            case Opcode::LoadInt:
                load_int();
                break;
            case Opcode::LoadFloat:
                load_float();
                break;
            case Opcode::LoadStrPtr:
                load_str_ptr();
                break;
            case Opcode::IntToFloat:
                int_to_float();
                break;
            case Opcode::IntToStrPtr:
                int_to_str_ptr();
                break;
            case Opcode::FloatToStrPtr:
                float_to_str_ptr();
                break;
            case Opcode::PushBool:
                push_bool();
                break;
            case Opcode::StoreBool:
                store_bool();
                break;
            case Opcode::LoadBool:
                load_bool();
                break;
            case Opcode::PrintBool:
                print_bool();
                break;
            case Opcode::IsEqInt:
                is_eq_int();
                break;
            case Opcode::IsEqFloat:
                is_eq_float();
                break;
            case Opcode::IsEqBool:
                is_eq_bool();
                break;
            case Opcode::IsNotEqInt:
                is_not_eq_int();
                break;
            case Opcode::IsNotEqFloat:
                is_not_eq_float();
                break;
            case Opcode::IsNotEqBool:
                is_not_eq_bool();
                break;
            case Opcode::IsGreaterOrEqFloat:
                is_greater_or_eq_float();
                break;
            case Opcode::IsGreaterOrEqInt:
                is_greater_or_eq_int();
                break;
            case Opcode::IsLessOrEqFloat:
                is_less_or_eq_float();
                break;
            case Opcode::IsLessOrEqInt:
                is_less_or_eq_int();
                break;
            case Opcode::IsGreaterFloat:
                is_greater_float();
                break;
            case Opcode::IsGreaterInt:
                is_greater_int();
                break;
            case Opcode::IsLessFloat:
                is_less_float();
                break;
            case Opcode::IsLessInt:
                is_less_int();
                break;
            case Opcode::JmpIfFalse:
                jump_if_false();
                break;
            case Opcode::Jmp:
                jump();
                break;
            case Opcode::JmpBack:
                jump_back();
                break;
            case Opcode::PopStack:
                pop_stack();
                break;
            default:
                throw InterpreterError("Invalid opcode");
        }
    }
}

void Interpreter::run() {
    Header header = read_header();

    jump_forwards(header.code_offset);

    run_code();
}
