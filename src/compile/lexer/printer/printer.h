#pragma once

#include <vector>
#include "compile/token.h"

void print_tokens(std::ostream& out, const std::vector<Token>& tokens, const std::string& source);
