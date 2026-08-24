#include "printer.h"

#include <cstring>
#include <string>

namespace {

bool is_const_ptr(uint64_t value) {
    return value & (1ULL << 63);
}

std::string read_const_string(
    const std::vector<uint8_t>& source,
    uint32_t offset
) {
    if (offset >= source.size()) {
        return {};
    }

    const uint8_t type = source[offset];
    (void)type;

    if (offset + 3 > source.size()) {
        return {};
    }

    uint16_t size;

    std::memcpy(
        &size,
        source.data() + offset + sizeof(type),
        sizeof(size)
    );

    const size_t string_start =
        offset +
        sizeof(type) +
        sizeof(size);

    if (string_start + size > source.size()) {
        return {};
    }

    std::string value(
        size,
        '\0'
    );

    std::memcpy(
        value.data(),
        source.data() + string_start,
        size
    );

    return value;
}

void print_value(
    std::ostream& out,
    uint64_t value,
    const std::vector<uint8_t>& source
) {
    if (is_const_ptr(value)) {
        const uint32_t offset =
            static_cast<uint32_t>(value);

        if (offset >= source.size()) {
            out
                << "invalid-const-ptr("
                << offset
                << ")";

            return;
        }

        out
            << "const-string(\""
            << read_const_string(
                source,
                offset
            )
            << "\")";

        return;
    }

    const uint32_t int_value =
        static_cast<uint32_t>(value);

    double float_value;

    std::memcpy(
        &float_value,
        &value,
        sizeof(float_value)
    );

    const bool bool_value =
        value != 0;

    out
        << "raw="
        << value
        << " | int="
        << int_value
        << " | float="
        << float_value
        << " | bool="
        << (bool_value ? "true" : "false");

    if (value != 0 &&
        value % alignof(std::string) == 0) {
        const auto* string_ptr =
            reinterpret_cast<const std::string*>(
                value
            );

        out
            << " | string-ptr=\""
            << *string_ptr
            << '"';
    }
}

void print_stack(
    std::ostream& out,
    const char* name,
    const Stack& stack,
    const std::vector<uint8_t>& source
) {
    out
        << "  "
        << name
        << ":\n";

    if (stack.size() == 0) {
        out << "    (empty)\n";
        return;
    }

    for (size_t i = 0; i < stack.size(); ++i) {
        out
            << "    ["
            << i
            << "] ";

        print_value(
            out,
            stack.stack_at(i),
            source
        );

        out << '\n';
    }
}

} // namespace

void print_stacks(
    std::ostream& out,
    const Stack& opstack,
    const Stack& stack,
    const std::vector<uint8_t>& source
) {
    print_stack(
        out,
        "operand stack",
        opstack,
        source
    );

    print_stack(
        out,
        "variable stack",
        stack,
        source
    );
}
