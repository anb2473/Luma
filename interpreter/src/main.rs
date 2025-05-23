mod parser_core {
    pub mod lexer;
    pub mod parser;
    pub mod tokenized;
    pub mod value;
    pub mod ast;
}

mod executer {
    pub mod runtime;
    pub mod interpreter;
}

fn main() {
    let mut lexer = parser_core::lexer::Lexer::new("C:\\Users\\austi\\projects\\Luma\\test.luma".to_string());
    lexer.run();

    let parser = parser_core::parser::Parser::new(lexer);
    let ast = parser.run();

    let interpreter = executer::interpreter::Interpreter::new(ast);
    interpreter.run();
}
