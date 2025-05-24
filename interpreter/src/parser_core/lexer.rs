use std::io::{self, Error, Read};
use std::fs::{self, File}; 
use std::path::{Path, PathBuf};

use crate::parser_core::value;
use crate::parser_core::value::CastTo;
use crate::parser_core::tokenized;
use std::collections::HashMap;

// **GOAL:** Read file contents, split the file contents into a Vec of lines, for each line split the line by its parts, and insert types where necessary

fn read_file(file_path: &str) -> Result<String, io::Error> {
    let path = Path::new(file_path);
    let mut file = File::open(path)?; // Open the file, propagate errors

    let mut contents = String::new();
    file.read_to_string(&mut contents)?; // Read contents, propagate errors

    Ok(contents) // Return the contents if successful
}

pub struct Lexer {
    pub file_contents: String,
    pub tokenized_lines: tokenized::Tokenized,
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

    pub fn run(&mut self) {
        // **GOAL:** Loop through every line and convert to a TokenList
        let mut split_line = self.file_contents.split("\n");

        let mut next_line = split_line.next();

        while let Some(mut line) = next_line {
            line = match line.split("//").next() {
                Some(val) => val,
                None => panic!("Failed to isolate non comment section of line")
            };

            line = line.trim();
            
            if line == "" {
                next_line = split_line.next();
                continue;
            }

            // Manages all of the special characters in the language
            let actions = HashMap::from([
                ('+', tokenized::Verb::Add),
                ('-', tokenized::Verb::Sub),
                ('*', tokenized::Verb::Mult),
                ('/', tokenized::Verb::Div),
                ('=', tokenized::Verb::Set),
            ]);

            // Check for suffix in the last character
            let mut chars: Vec<char> = line.chars().collect();
            let suffix = if let Some(&last_char) = chars.last() {
                match last_char {
                    ';' => Some(tokenized::Suffix::Set),
                    _ => {
                        chars.push(last_char);  // Cancel out suffix removal to keep the suffix
                        Some(tokenized::Suffix::Return)
                    },
                }
            } else {
                None
            };

            // Remove the last character if it was a suffix
            if suffix.is_some() {
                chars.pop();
            }

            // Sliding Window approach: loop through each character in the line and reference it with the actions list, O(n) time complexity
            let mut slider = String::new();
            let mut token_list: Vec<tokenized::Token> = Vec::new();

            for character in chars {
                if let Some(action) = actions.get(&character) {
                    if !slider.trim().is_empty() {
                        token_list.push(tokenized::Token::Noun(value::Value::evaluate(slider.trim().to_string())));
                    }

                    token_list.push(tokenized::Token::Verb(action.clone()));

                    slider = String::new();
                    continue;
                }

                slider.push(character);
            }

            // Handle the last token if there is one
            if !slider.trim().is_empty() {
                token_list.push(tokenized::Token::Noun(value::Value::evaluate(slider.trim().to_string())));
            }

            // Add the token list to our lines
            self.tokenized_lines.lines.push(tokenized::TokenList {
                objects: token_list,
                suffix: suffix,
            });

            next_line = split_line.next();
        }
    }
}