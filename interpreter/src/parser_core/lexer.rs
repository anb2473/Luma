use std::io::{self, Error, Read};
use std::fs::{self, File}; 
use std::path::{Path, PathBuf};

use crate::parser_core::value;
use crate::parser_core::value::CastTo;
use crate::parser_core::tokenized;

// **GOAL:** Read file contents, split the file contents into a Vec of lines, for each line split the line by its parts, and insert types where necessary

fn read_file(file_path: &str) -> Result<String, io::Error> {
    let path = Path::new(file_path);
    let mut file = File::open(path)?; // Open the file, propagate errors

    let mut contents = String::new();
    file.read_to_string(&mut contents)?; // Read contents, propagate errors

    Ok(contents) // Return the contents if successful
}

pub struct Lexer {
    file_contents: String,
}

impl Lexer {
    pub fn new(file_path: String) -> Self {
        Lexer {
            file_contents: match read_file(file_path.as_str()) {
                Ok(val) => val,
                Err(err) => panic!("{}", err),
            },
            tokenized_lines: tokenized::Tokenized {
                lines: Vec::new(),
            },
        }
    }

    pub fn run(&self) {
        // **GOAL:** Loop through every line and convert to a TokenList
        let split_line = self.file_contents.split("\n");

        let next_line = split_line.next();

        while let Some(mut line) = next_line {
            line = line.trim();

            
        }
    }
}