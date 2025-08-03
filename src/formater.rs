use crate::Config;
use sqlparser::dialect::{dialect_from_str, GenericDialect};
use sqlparser::tokenizer::{Token, Tokenizer};
use sqlparser::keywords::Keyword;

#[derive(Debug, Clone, Copy)]
pub struct IndentationCount(usize, bool);

impl IndentationCount {
    pub fn new() -> IndentationCount {
        IndentationCount(0, false)
    }

    pub fn add(&mut self) {
        self.0 += 1;
    }

    pub fn skip_last(&mut self) {
        self.1 = true;
    }

    pub fn reset_skip(&mut self) {
        self.1 = false;
    }

    pub fn sub(&mut self) {
        self.0 = self.0.saturating_sub(1);
    }

    pub fn get(&self) -> usize {
        self.0
    }

    pub fn get_skip(&self) -> bool {
        self.1
    }
}

pub fn formater(config: Config, script: String) -> Result<String, String> {
    let dialect = dialect_from_str(&config.database)
        .unwrap_or(Box::new(GenericDialect::default()));

    let tokens = Tokenizer::new(dialect.as_ref(), &script)
        .tokenize()
        .map_err(|e| format!("{}", e))?;

    process_format(config, tokens)
}

fn process_format(config: Config, tokens: Vec<Token>) -> Result<String, String> {
    let mut indentation = IndentationCount::new();
    let mut result = String::new();
    let mut buffer = String::new();
    let mut index = 0usize;

    while index < tokens.len() {
        match &tokens[index] {
            Token::Word(word) => {
                match (word.keyword, config.keywords_case.as_str()) {
                    (Keyword::NoKeyword, _) => buffer.push_str(&word.value),
                    (_, "lowercase" | "lower") => buffer.push_str(&word.value.to_lowercase()),
                    (_, "uppercase" | "upper") => buffer.push_str(&word.value.to_uppercase()),
                    (_, case) => { return Err(format!("Unsupported case : {} in {:?}", case, config)) }
                }
            },
            Token::EOF => {},
            Token::Comma => {
                if config.linebreak_after_comma {
                    buffer.push_str(",\n");
                }
                else {
                    buffer.push_str(",");
                }
            },
            Token::SemiColon => {
                if config.linebreak_after_semicolon {
                    buffer.push_str(";\n")
                }
            }
            Token::LParen => {
                if config.linebreak_after_left_parenthesis {
                    buffer.push_str("(\n");
                    indentation.add();
                    indentation.skip_last();
                }
                else {
                    buffer.push('(');
                }
            },
            Token::LBrace => {
                if config.linebreak_after_left_brace {
                    buffer.push_str("{\n");
                    indentation.add();
                    indentation.skip_last();
                }
                else {
                    buffer.push('{')
                }
            },
            Token::LBracket => {
                if config.linebreak_after_left_bracket {
                    buffer.push_str("[\n");
                    indentation.add();
                    indentation.skip_last();
                }
                else {
                    buffer.push('[');
                }
            },
            Token::RParen => {
                if config.linebreak_after_left_parenthesis {
                    indentation.sub();
                }
                buffer.push(')');
            },
            Token::RBrace => {
                if config.linebreak_after_left_brace {
                    indentation.sub()
                }
                buffer.push('}')
            },
            Token::RBracket => {
                if config.linebreak_after_left_bracket {
                    indentation.sub()
                }
                buffer.push(']');
            },
            other_token => {
                buffer.push_str(&format!("{}", other_token));
            }
        }

        let indentation_string: String;
        if indentation.get_skip() {
            indentation_string = "\t".repeat(indentation.get().saturating_sub(1));
        }
        else {
            indentation_string = "\t".repeat(indentation.get());
        }

        result.push_str(&indentation_string);
        result.push_str(&buffer);
        
        buffer.clear();
        index += 1;
    }

    Ok(result)
}
