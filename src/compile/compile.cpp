#include <chrono>
#include <filesystem>
#include <fstream>
#include <iomanip>
#include <iostream>
#include <string>
#include <vector>

#include "error.h"
#include "loader/loader.h"
#include "lexer/lexer.h"
#include "lexer/printer/printer.h"
#include "token.h"
#include "parser/parser.h"
#include "parser/printer/printer.h"
#include "semantic_analysis/SemanticAnalysis.h"
#include "bytecode/bytecode.h"
#include "bytecode/printer/printer.h"
#include "compiler_options.h"

using Clock = std::chrono::steady_clock;

namespace {

double duration_ms(
    Clock::time_point start,
    Clock::time_point end
) {
    return std::chrono::duration<double, std::milli>(
        end - start
    ).count();
}

void print_duration(
    const char* name,
    double duration,
    std::ostream& out
) {
    out << std::fixed
        << std::setprecision(5)
        << name
        << ": "
        << duration
        << " ms\n";
}

std::vector<Token> lex_source(
    const std::string& source
) {
    Lexer lexer(source);
    return lexer.tokenize();
}

ASTNode parse_tokens(
    std::vector<Token> tokens,
    const std::string& source
) {
    Parser parser(
        std::move(tokens),
        source
    );

    return parser.parse();
}

void analyze_ast(
    ASTNode& ast
) {
    SemanticAnalysis analyzer(ast);
    analyzer.analyze();
}

ASTNode compile_source(
    const std::string& source,
    const CompilerOptions& options,
    std::ostream& diagnostic_out,
    bool write_logs
) {
    if (write_logs &&
        (options.print_tokens ||
         options.print_ast ||
         options.compiletime)) {
        std::filesystem::create_directories("logs");
    }

    const auto total_start = Clock::now();

    const auto lexer_start = Clock::now();

    std::vector<Token> tokens =
        lex_source(source);

    const auto lexer_end = Clock::now();

    if (options.print_tokens) {
        if (write_logs) {
            std::ofstream log("logs/lexer.txt");

            if (!log) {
                throw std::runtime_error(
                    "Could not open logs/lexer.txt"
                );
            }

            print_tokens(
                log,
                tokens,
                source
            );
        } else {
            print_tokens(
                diagnostic_out,
                tokens,
                source
            );

            diagnostic_out << '\n';
        }
    }

    const auto parser_start = Clock::now();

    ASTNode ast =
        parse_tokens(
            std::move(tokens),
            source
        );

    const auto parser_end = Clock::now();

    if (options.print_ast) {
        if (write_logs) {
            std::ofstream log("logs/parser.txt");

            if (!log) {
                throw std::runtime_error(
                    "Could not open logs/parser.txt"
                );
            }

            print_ast(
                log,
                ast
            );
        } else {
            print_ast(
                diagnostic_out,
                ast
            );

            diagnostic_out << '\n';
        }
    }

    const auto semantic_start = Clock::now();

    analyze_ast(ast);

    const auto semantic_end = Clock::now();

    const auto total_end = Clock::now();

    if (options.compiletime) {
        if (write_logs) {
            std::ofstream compiletime_log(
                "logs/compiletime.txt"
            );

            if (!compiletime_log) {
                throw std::runtime_error(
                    "Could not open logs/compiletime.txt"
                );
            }

            print_duration(
                "Lexer",
                duration_ms(
                    lexer_start,
                    lexer_end
                ),
                compiletime_log
            );

            print_duration(
                "Parser",
                duration_ms(
                    parser_start,
                    parser_end
                ),
                compiletime_log
            );

            print_duration(
                "Semantic Analysis",
                duration_ms(
                    semantic_start,
                    semantic_end
                ),
                compiletime_log
            );

            print_duration(
                "Total",
                duration_ms(
                    total_start,
                    total_end
                ),
                compiletime_log
            );
        } else {
            print_duration(
                "Lexer",
                duration_ms(
                    lexer_start,
                    lexer_end
                ),
                diagnostic_out
            );

            print_duration(
                "Parser",
                duration_ms(
                    parser_start,
                    parser_end
                ),
                diagnostic_out
            );

            print_duration(
                "Semantic Analysis",
                duration_ms(
                    semantic_start,
                    semantic_end
                ),
                diagnostic_out
            );

            print_duration(
                "Total",
                duration_ms(
                    total_start,
                    total_end
                ),
                diagnostic_out
            );

            diagnostic_out << '\n';
        }
    }

    return ast;
}

}

void compile_file(
    const std::string& path,
    const CompilerOptions& options
) {
    const std::string source =
        load_in(path);

    ASTNode ast =
        compile_source(
            source,
            options,
            std::cout,
            true
        );

    Program& program = std::get<Program>(ast.data);

    Compiler compiler(program, {});

    std::vector<LoopContext> loop_context;

    BytecodeWriter writer =
        compiler.to_bytecode();

    if (options.print_bytecode) {
        std::ofstream log(
            "logs/bytecode.txt"
        );

        if (!log) {
            throw std::runtime_error(
                "Could not open logs/bytecode.txt"
            );
        }

        BytecodePrinter printer(
            log,
            writer
        );

        printer.print();
    }

    std::ofstream output =
        load_out(path);

    if (!output) {
        throw std::runtime_error(
            "Could not open output file"
        );
    }

    writer.flush_file(output);
}

CompilerOptions parse_options(
    int argc,
    char* argv[]
) {
    CompilerOptions options;

    for (int i = 2; i < argc; ++i) {
        const std::string arg = argv[i];

        if (arg == "compiletime") {
            options.compiletime = true;
        } else if (arg == "tokens") {
            options.print_tokens = true;
        } else if (arg == "ast") {
            options.print_ast = true;
        } else if (arg == "bytecode") {
            options.print_bytecode = true;
        } else {
            throw std::runtime_error(
                "Unknown option: " + arg
            );
        }
    }

    return options;
}

void print_usage(
    const char* program
) {
    std::cerr
        << "Usage:\n"
        << "  " << program << " <file> [options]\n\n"
        << "Options:\n"
        << "  compiletime  Show compilation times\n"
        << "  tokens       Show tokens\n"
        << "  ast          Show AST\n"
        << "  bytecode     Show bytecode\n";
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

        const CompilerOptions options =
            parse_options(
                argc,
                argv
            );

        compile_file(
            path,
            options
        );

        return 0;

    } catch (const ParserError& e) {
        std::cerr
            << "Parser Error: "
            << e.what()
            << '\n';

        return 1;

    } catch (const LexerError& e) {
        std::cerr
            << "Lexer Error: "
            << e.what()
            << '\n';

        return 1;

    } catch (const SemanticAnalysisError& e) {
        std::cerr
            << "Semantic Error: "
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