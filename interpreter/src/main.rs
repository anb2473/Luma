mod parser_core {
    pub mod lexer;
    pub mod value;  // Expose value to lexer so that the lexer can import the value
    pub mod tokenized;
}

fn main() {
    let mut lexer = parser_core::lexer::Lexer::new("C:\\Users\\austi\\projects\\Luma\\test.luma".to_string());
    parser_core::lexer::Lexer::run(&mut lexer)
    ;
    println!("{:?} {:?}", lexer.tokenized_lines, lexer.file_contents);
}
