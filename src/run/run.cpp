#include <chrono>
#include <filesystem>
#include <fstream>
#include <iostream>
#include <iterator>
#include <string>
#include <vector>

#include "interpreter/interpreter.h"
#include "run/error.h"
#include "run_options.h"
#include "interpreter/printer/printer.h"

namespace {

RunOptions parse_options(
    int argc,
    char* argv[]
) {
    RunOptions options;

    for (int i = 2; i < argc; ++i) {
        const std::string arg =
            argv[i];

        if (arg == "dump-stacks") {
            options.dump_stacks = true;
        } else if (arg == "runtime") {
            options.runtime = true;
        } else {
            throw std::runtime_error(
                "Unknown option: " + arg
            );
        }
    }

    return options;
}

void run_file(
    const std::string& path,
    const RunOptions& options
) {
    std::ifstream input(
        path,
        std::ios::binary
    );

    if (!input) {
        throw std::runtime_error(
            "Could not open bytecode file: " +
            path
        );
    }

    std::vector<uint8_t> bytecode(
        std::istreambuf_iterator<char>(input),
        {}
    );

    Interpreter interpreter(
        std::move(bytecode),
        std::cout
    );

    std::chrono::steady_clock::time_point start;

    if (options.runtime) {
        start = std::chrono::steady_clock::now();
    }

    interpreter.run();

    if (options.runtime) {
        const auto end =
            std::chrono::steady_clock::now();

        const std::chrono::duration<double, std::milli> elapsed =
            end - start;

        std::filesystem::create_directories("logs");

        std::ofstream log(
            "logs/runtime.logs"
        );

        if (!log) {
            throw std::runtime_error(
                "Could not open logs/runtime.logs"
            );
        }

        log
            << "Run: "
            << elapsed.count()
            << " ms\n";
    }

    if (!options.dump_stacks) {
        return;
    }

    std::filesystem::create_directories("logs");

    std::ofstream log(
        "logs/stacks.txt"
    );

    if (!log) {
        throw std::runtime_error(
            "Could not open logs/stacks.txt"
        );
    }

    log << "Interpreter state:\n";

    print_stacks(
        log,
        interpreter.get_opstack(),
        interpreter.get_stack(),
        interpreter.get_source()
    );
}

void print_usage(
    const char* program
) {
    std::cerr
        << "Usage:\n"
        << "  " << program << " <file> [options]\n\n"
        << "Options:\n"
        << "  --dump-stacks    Dump interpreter stacks to logs/stacks.txt\n"
        << "  --runtime        Record interpreter runtime in logs/runtime.logs\n";
}

}

int main(
    int argc,
    char* argv[]
) {
    try {
        if (argc < 2) {
            print_usage(argv[0]);
            return 1;
        }

        const std::string path =
            argv[1];

        const RunOptions options =
            parse_options(
                argc,
                argv
            );

        run_file(
            path,
            options
        );

        return 0;

    } catch (const InterpreterError& e) {
        std::cerr
            << "Interpreter Error: "
            << e.what()
            << '\n';

        return 1;

    } catch (const std::exception& e) {
        std::cerr
            << "Error: "
            << e.what()
            << '\n';

        return 1;
    }
}
