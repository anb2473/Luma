#pragma once

struct CompilerOptions {
    bool print_tokens = false;
    bool print_ast = false;
    bool print_bytecode = false;
    bool compiletime = false;
    bool repl = false;
};
