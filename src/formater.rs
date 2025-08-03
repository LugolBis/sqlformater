use crate::Config;
use sqlparser::dialect::{dialect_from_str, GenericDialect};
use sqlparser::tokenizer::{Token, Tokenizer};
use sqlparser::keywords::Keyword;

#[derive(Debug, Clone, Copy)]
pub struct IndentationCount(usize);

impl IndentationCount {
    pub fn new() -> IndentationCount {
        IndentationCount(0)
    }

    pub fn add(&mut self) {
        self.0 += 1;
    }

    pub fn sub(&mut self) {
        self.0 = self.0.saturating_sub(1);
    }

    #[allow(unused)]
    pub fn get(&self) -> usize {
        self.0
    }

    pub fn get_string(&self, skip: Option<usize>) -> String {
        if let Some(number) = skip {
            "\t".repeat(self.0.saturating_sub(number))
        }
        else {
            "\t".repeat(self.0)
        }
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
    let mut buffer = String::new();
    let mut result = String::new();
    let mut index = 0usize;

    while index < tokens.len() {
        match &tokens[index] {
            Token::Word(word) => {
                if result.ends_with("\n") {
                    buffer.push_str(&indentation.get_string(None));
                }

                let value: String;
                match (word.keyword, config.keywords_case.as_str()) {
                    (Keyword::NoKeyword, _) => value = word.value.clone(),
                    (_, "lowercase" | "lower") => value = word.value.to_lowercase(),
                    (_, "uppercase" | "upper") => value = word.value.to_uppercase(),
                    (_, case) => { return Err(format!("Unsupported case : {} in {:?}", case, config)) }
                }

                if config.linebreak_before_keywords.contains(&value) {
                    buffer.push('\n');
                }

                buffer.push_str(&value);

                if config.linebreak_after_keywords.contains(&value) {
                    buffer.push('\n');
                }
            },
            Token::EOF => {},
            Token::Comma => {
                if config.linebreak_after_comma {
                    buffer.push_str(",\n");
                }
                else {
                    buffer.push(',');
                }
            },
            Token::SemiColon => {
                if config.linebreak_after_semicolon {
                    buffer.push_str(";\n")
                }
                else {
                    buffer.push(';');
                }
            }
            Token::LParen => {
                if config.linebreak_after_lparenthesis {
                    buffer.push_str("(\n");
                }
                else {
                    buffer.push('(');
                }

                if config.indentation_parenthesis {
                    indentation.add();
                }
            },
            Token::LBrace => {
                if config.linebreak_after_lbrace {
                    buffer.push_str("{\n");
                }
                else {
                    buffer.push('{');
                }

                if config.indentation_braces {
                    indentation.add();
                }
            },
            Token::LBracket => {
                if config.linebreak_after_lbracket {
                    buffer.push_str("[\n");
                }
                else {
                    buffer.push('[');
                }

                if config.indentation_brackets {
                    indentation.add();
                }
            },
            Token::RParen => {
                if config.linebreak_after_lparenthesis {
                    buffer.push('\n');
                }

                if config.indentation_parenthesis {
                    buffer.push_str(&indentation.get_string(Some(1)));
                    indentation.sub();
                }
                buffer.push(')');
            },
            Token::RBrace => {
                if config.linebreak_after_lbrace {
                    buffer.push('\n');
                }

                if config.indentation_braces {
                    buffer.push_str(&indentation.get_string(Some(1)));
                    indentation.sub();
                }
                buffer.push('}');
            },
            Token::RBracket => {
                if config.linebreak_after_lbracket {
                    buffer.push('\n');
                }

                if config.indentation_brackets {
                    buffer.push_str(&indentation.get_string(Some(1)));
                    indentation.sub();
                }
                buffer.push(']');
            },
            Token::Whitespace(whitespace) => {
                if !result.ends_with("\n") && !result.ends_with("\t") {
                    buffer.push_str(&format!("{}", whitespace));
                }
            }
            other_token => {
                if result.ends_with("\n") {
                    buffer.push_str(&indentation.get_string(None));
                }
                buffer.push_str(&format!("{}", other_token));
            }
        }

        result.push_str(&buffer);
        buffer.clear();
        index += 1;
    }

    Ok(result)
}
