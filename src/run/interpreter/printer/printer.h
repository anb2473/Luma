#pragma once

#include <cstdint>
#include <ostream>
#include <vector>

#include "../interpreter.h"

void print_stacks(
    std::ostream& out,
    const Stack& opstack,
    const Stack& stack,
    const std::vector<uint8_t>& source
);
