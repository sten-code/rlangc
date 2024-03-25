extern crate phf;
use phf::phf_map;
use std::fmt;

pub static KEYWORDS: phf::Map<&'static str, TokenType> = phf_map! {
    "fn" => TokenType::Fn,
    "typedef" => TokenType::TypeDef,
    "struct" => TokenType::Struct,
};

#[derive(Debug, Clone, PartialEq)]
pub enum TokenType {
    Identifier,
    Integer,
    Float,
    Add,
    Fn,
    TypeDef,
    Struct,
    OpenBrace,
    CloseBrace,
    Equals,
    Semicolon,
    Comma,
}

#[derive(Debug)]
pub struct Token {
    pub token_type: TokenType,
    pub value: String,
    pub start_index: usize,
    pub end_index: usize,
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "[{:?}: {}] at {}-{}",
            self.token_type, self.value, self.start_index, self.end_index
        )
    }
}

#[derive(Debug)]
pub enum LexerError {
    IllegalCharacter,
    InvalidFloat,
}

pub fn lex(script: String) -> Result<Vec<Token>, LexerError> {
    let mut tokens = Vec::new();

    let mut i = 0;
    while i < script.len() {
        let c = script.chars().nth(i).unwrap();

        if c.is_whitespace() {
            i += 1;
            continue;
        }

        if c == ',' {
            tokens.push(Token {
                token_type: TokenType::Comma,
                value: String::from(","),
                start_index: i,
                end_index: i,
            })
        } else if c == ';' {
            tokens.push(Token {
                token_type: TokenType::Semicolon,
                value: String::from(";"),
                start_index: i,
                end_index: i,
            })
        } else if c == '=' {
            tokens.push(Token {
                token_type: TokenType::Equals,
                value: String::from("="),
                start_index: i,
                end_index: i,
            })
        } else if c == '+' {
            tokens.push(Token {
                token_type: TokenType::Add,
                value: String::from("+"),
                start_index: i,
                end_index: i,
            });
        } else if c == '{' {
            tokens.push(Token {
                token_type: TokenType::OpenBrace,
                value: String::from("{"),
                start_index: i,
                end_index: i,
            });
        } else if c == '}' {
            tokens.push(Token {
                token_type: TokenType::CloseBrace,
                value: String::from("}"),
                start_index: i,
                end_index: i,
            });
        } else if c.is_alphabetic() {
            match parse_word(i, &script) {
                Ok(result) => {
                    i = result.0;
                    tokens.push(result.1);
                }
                Err(err) => return Err(err),
            }
        } else if c.is_digit(10) {
            match parse_number(i, &script) {
                Ok(result) => {
                    i = result.0;
                    tokens.push(result.1);
                }
                Err(err) => return Err(err),
            }
        } else {
            println!("Illegal character: {}", c);
            return Err(LexerError::IllegalCharacter);
        }

        i += 1;
    }

    Ok(tokens)
}

fn parse_word(index: usize, script: &str) -> Result<(usize, Token), LexerError> {
    let mut word = String::from("");
    let mut end = 0;

    for (i, c) in script.char_indices().skip(index) {
        if c.is_alphanumeric() {
            word.push(c);
        } else {
            end = i - 1;
            break;
        }
    }

    Ok((
        end,
        Token {
            token_type: if KEYWORDS.contains_key(&word) {
                KEYWORDS.get(&word).unwrap().clone()
            } else {
                TokenType::Identifier
            },
            value: word,
            start_index: index,
            end_index: end,
        },
    ))
}

fn parse_number(index: usize, script: &str) -> Result<(usize, Token), LexerError> {
    let mut number = String::from("");
    let mut end = 0;
    let mut dot_count = 0;
    for (i, c) in script.char_indices().skip(index) {
        if c == '.' {
            if dot_count == 0 {
                dot_count += 1;
            } else {
                return Err(LexerError::InvalidFloat);
            }
        } else if !c.is_digit(10) {
            end = i - 1;
            break;
        }
        number.push(c);
    }
    Ok((
        end,
        Token {
            token_type: if dot_count == 0 {
                TokenType::Integer
            } else {
                TokenType::Float
            },
            value: number,
            start_index: index,
            end_index: end,
        },
    ))
}
