mod parser_core {
    pub mod lexer;
    pub mod value;  // Expose value to lexer so that the lexer can import the value
    pub mod tokenized;
    pub mod parser;
    pub mod ast;
}

pub mod executer {
    pub mod interpreter;
    pub mod runtime {
        pub mod executer;
    }
}

fn main() {
    let mut lexer = parser_core::lexer::Lexer::new("C:\\Users\\austi\\projects\\Luma\\test.luma".to_string());
    parser_core::lexer::Lexer::run(&mut lexer);
    
    let parser = parser_core::parser::Parser::new(lexer);
    let AST = parser_core::parser::Parser::run(&parser);

    let interpreter = executer::interpreter::Interpreter::new(AST);
    executer::interpreter::Interpreter::run(&interpreter);
}
