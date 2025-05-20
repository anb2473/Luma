mod parser_core {
    pub mod lexer;
    pub mod value;  // Expose value to lexer so that the lexer can import the value
}

fn main() {
    let lexer = parser_core::lexer::Lexer::new("C:\\Users\\austi\\projects\\Luma\\test.luma".to_string());
    parser_core::lexer::Lexer::run(&lexer);
}
