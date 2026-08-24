#pragma once

#include <stdexcept>
#include <string>

class LexerError : public std::runtime_error {
public:
    LexerError(const std::string& message)
        : std::runtime_error(message) {}
};

class ParserError : public std::runtime_error {
public:
    ParserError(const std::string& message)
        : std::runtime_error(message) {}
};

class TokenStreamError : public std::runtime_error {
public:
    TokenStreamError(const std::string& message)
        : std::runtime_error(message) {}
};

class SemanticAnalysisError : public std::runtime_error {
public:
    SemanticAnalysisError(const std::string& message)
    : std::runtime_error(message) {}
};

class BytecodeError : public std::runtime_error {
public:
    BytecodeError(const std::string& message)
        : std::runtime_error(message) {}
};