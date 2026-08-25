#pragma once

#include <vector>
#include <sys/mman.h>
#include <cstddef>
#include <cstdint>
#include <stdexcept>

struct Header {
    uint32_t code_offset;
};

class Stack {
    private:
        uint64_t* stack_pointer = nullptr;
        uint64_t* stack_base = nullptr;
        uint64_t* stack_end = nullptr;
        size_t stack_size;

    public:
        Stack(size_t stack_size)
            : stack_size(stack_size) {

            stack_base = static_cast<uint64_t*>(
                mmap(
                    nullptr,
                    stack_size * sizeof(uint64_t),
                    PROT_READ | PROT_WRITE,
                    MAP_PRIVATE | MAP_ANONYMOUS,
                    -1,
                    0
                )
            );

            if (stack_base == MAP_FAILED) {
                stack_base = nullptr;
                throw std::runtime_error("Could not allocate interpreter stack");
            }
            stack_end = stack_base + stack_size;
            stack_pointer = stack_end;
        }
        ~Stack() {
            if (stack_base != nullptr) {
                munmap(
                    stack_base,
                    stack_size * sizeof(uint64_t)
                );
            }
        }
        void push_u64(uint64_t value);
        void push_f64(double value);
        uint64_t pop_u64();
        double pop_f64();
        uint64_t stack_at(size_t offset) const;
        void set_u64(uint64_t value, uint16_t offset);
        void set_f64(double value, uint16_t offset);
        size_t size() const {
            return stack_end - stack_pointer;
        }
};

class Interpreter {
    private:
        std::vector<std::uint8_t> source;
        Header read_header();
        void run_code();
        size_t pos = 0;
        uint8_t read_byte();
        uint32_t read_u32();
        double read_f64();
        uint16_t read_u16();
        std::string read_const_str(int32_t offset);
        void add_str_ptr();
        void jump_forwards(size_t dist);
        void push_int();
        void push_float();
        void push_const();
        void push_bool();
        void add_int();
        void print_bool();
        void store_bool();
        void is_eq_int();
        void is_eq_bool();
        void is_eq_float();
        void is_less_float();
        void is_less_int();
        void is_greater_int();
        void is_greater_float();
        void is_greater_or_eq_int();
        void is_greater_or_eq_float();
        void is_less_or_eq_float();
        void is_less_or_eq_int();
        void jump_if_false();
        void add_float();
        void load_bool();
        void sub_int();

        void sub_unary_int();

        void sub_float();

        void sub_unary_float();

        void mult_int();
        void mult_float();
        void div_int();
        void div_float();
        void store_int();
        void store_ptr();
        void store_float();
        void store_const();
        Stack stack;
        void pop_stack();

        void pop_str_ptr();

        void sleep_int();

        void sleep_float();

        Stack opstack;
        void print_int();
        void print_float();
        void print_str_ptr();
        std::ostream& out;
        void load_int();
        void load_float();
        void load_str_ptr();
        void float_to_str_ptr();
        void int_to_str_ptr();
        void int_to_float();
        void is_not_eq_bool();
        void is_not_eq_int();
        void is_not_eq_float();
        void jump();
        void jump_backwards(size_t dist);
        void jump_back();
        void jump_through();
    public:
        Interpreter(std::vector<std::uint8_t> source, std::ostream& out)
            : source(source), stack(1024*1024), opstack(1024), out(out) {}
        void run();
        const Stack& get_opstack() const {
            return opstack;
        }

        const Stack& get_stack() const {
            return stack;
        }

        const std::vector<uint8_t>& get_source() const {
            return source;
        }
};
