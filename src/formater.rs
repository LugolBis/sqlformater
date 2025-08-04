use crate::Settings;
use sqlparser::dialect::{dialect_from_str, GenericDialect};
use sqlparser::tokenizer::{Token, Tokenizer};
use sqlparser::keywords::Keyword;

#[derive(Debug, Clone)]
pub struct IndentationCount(usize, String);

impl IndentationCount {
    pub fn new(value: &str) -> IndentationCount {
        IndentationCount(0, String::from(value))
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
            self.1.repeat(self.0.saturating_sub(number))
        }
        else {
            self.1.repeat(self.0)
        }
    }
}

pub fn formater(settings: Settings, script: String) -> Result<String, String> {
    let dialect = dialect_from_str(&settings.database)
        .unwrap_or(Box::new(GenericDialect::default()));

    let tokens = Tokenizer::new(dialect.as_ref(), &script)
        .tokenize()
        .map_err(|e| format!("{}", e))?;

    process_format(settings, tokens)
}

fn process_format(settings: Settings, tokens: Vec<Token>) -> Result<String, String> {
    let mut indentation = IndentationCount::new(&settings.tabulation_format);
    let mut buffer = String::new();
    let mut result = String::new();
    let mut index = 0usize;

    while index < tokens.len() {
        match &tokens[index] {
            Token::Word(word) => {
                match (result.ends_with("\n"), settings.indentation_clauses, word.keyword) {
                    (true, true, Keyword::SELECT) => {
                        buffer.push_str(&indentation.get_string(None));
                        indentation.add();
                    },
                    (false, true, Keyword::SELECT) => {
                        buffer.push_str(&indentation.get_string(None));
                        indentation.add();
                    },
                    (true, true, Keyword::FROM | Keyword::WHERE) => {
                        buffer.push_str(&indentation.get_string(Some(1)));
                    },
                    (false, true, Keyword::FROM | Keyword::WHERE) => {
                        buffer.push('\n');
                        buffer.push_str(&indentation.get_string(Some(1)));
                    },
                    (true, _, _) => {
                        buffer.push_str(&indentation.get_string(None));
                    },
                    (false, _, _) => {}
                }

                let case = settings.keywords_case.as_str();
                if let (Keyword::NoKeyword, _) = (word.keyword, case) {
                    buffer.push_str(&word.value.clone())
                }
                else {
                    let value: String;
                    match case {
                        "lowercase" | "lower" => value = word.value.to_lowercase(),
                        "uppercase" | "upper" => value = word.value.to_uppercase(),
                        _ => {return Err(format!("Unsupported case : {} in settings :\n{:?}", case, settings))}
                    }

                    if !result.ends_with("\n")
                        && (settings.linebreak_before_keywords.contains(&value) 
                            || settings.linebreak_before_keywords.contains(&"*".to_string()))
                    {
                        buffer.push('\n');
                    }

                    buffer.push_str(&value);

                    if !result.ends_with("\n")
                        && (settings.linebreak_after_keywords.contains(&value)
                            || settings.linebreak_after_keywords.contains(&"*".to_string()))
                    {
                        buffer.push('\n');
                    }
                }
            },
            Token::EOF => {},
            Token::Comma => {
                if settings.linebreak_after_comma {
                    buffer.push_str(",\n");
                }
                else {
                    buffer.push(',');
                }
            },
            Token::SemiColon => {
                if settings.linebreak_after_semicolon {
                    buffer.push_str(";\n")
                }
                else {
                    buffer.push(';');
                }
            }
            Token::LParen => {
                if settings.linebreak_after_lparenthesis {
                    buffer.push_str("(\n");
                }
                else {
                    buffer.push('(');
                }

                if settings.indentation_parenthesis {
                    indentation.add();
                }
            },
            Token::LBrace => {
                if settings.linebreak_after_lbrace {
                    buffer.push_str("{\n");
                }
                else {
                    buffer.push('{');
                }

                if settings.indentation_braces {
                    indentation.add();
                }
            },
            Token::LBracket => {
                if settings.linebreak_after_lbracket {
                    buffer.push_str("[\n");
                }
                else {
                    buffer.push('[');
                }

                if settings.indentation_brackets {
                    indentation.add();
                }
            },
            Token::RParen => {
                if settings.linebreak_after_lparenthesis {
                    buffer.push('\n');
                }

                if settings.indentation_parenthesis {
                    buffer.push_str(&indentation.get_string(Some(1)));
                    indentation.sub();
                }
                buffer.push(')');
            },
            Token::RBrace => {
                if settings.linebreak_after_lbrace {
                    buffer.push('\n');
                }

                if settings.indentation_braces {
                    buffer.push_str(&indentation.get_string(Some(1)));
                    indentation.sub();
                }
                buffer.push('}');
            },
            Token::RBracket => {
                if settings.linebreak_after_lbracket {
                    buffer.push('\n');
                }

                if settings.indentation_brackets {
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
