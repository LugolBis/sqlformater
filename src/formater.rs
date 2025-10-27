use std::fs::OpenOptions;
use std::io::{Read, Write};
use std::path::PathBuf;

use crate::settings::Settings;
use sqlparser::dialect::{GenericDialect, dialect_from_str};
use sqlparser::keywords::Keyword;
use sqlparser::tokenizer::{Token, Tokenizer};

struct Indentation(usize, usize, String);

impl Indentation {
    fn new(value: &str) -> Self {
        Indentation(0, 1, String::from(value))
    }

    fn update(&mut self, settings: &Settings, word: &str) -> String {
        let (start, end) = (&settings.indentation_start, &settings.indentation_end);

        match (start.contains(word), end.contains(word)) {
            (true, true) => {
                self.1 = self.0;
                self.0 = self.0.saturating_sub(1);
            }
            (true, false) => {
                self.1 += 1;
            }
            (false, true) => {
                self.0 = self.0.saturating_sub(1);
                self.1 = self.1.saturating_sub(2);
            }
            (false, false) => {
                if self.0 == 0 {
                    self.0 += 1;
                }
            }
        }
        self.get_string(word)
    }

    fn get_string(&mut self, word: &str) -> String {
        let indentation = self.2.repeat(self.0);
        self.0 = self.1;
        format!("{}{} ", indentation, word)
    }
}

pub fn formater(settings: &Settings, script_path: PathBuf) -> Result<(), String> {
    let mut script = String::new();
    let mut file = OpenOptions::new()
        .read(true)
        .open(&script_path)
        .map_err(|e| format!("{}", e))?;

    let _ = file
        .read_to_string(&mut script)
        .map_err(|e| format!("{}", e))?;

    let dialect =
        dialect_from_str(&settings.database).unwrap_or(Box::new(GenericDialect::default()));

    let tokens = Tokenizer::new(dialect.as_ref(), &script)
        .tokenize()
        .map_err(|e| format!("{}", e))?;

    let tokens = tokens
        .iter()
        .filter(|t| !matches!(t, Token::Whitespace(..)))
        .collect::<Vec<&Token>>();

    match process_format(settings, tokens) {
        Ok(formated_script) => {
            let mut file = OpenOptions::new()
                .truncate(true)
                .write(true)
                .open(&script_path)
                .map_err(|e| format!("{}", e))?;

            let _ = file
                .write(formated_script.as_bytes())
                .map_err(|e| format!("{}", e))?;
            Ok(())
        }
        Err(error) => Err(error),
    }
}

fn process_format(settings: &Settings, tokens: Vec<&Token>) -> Result<String, String> {
    let mut indentation = Indentation::new(&settings.tabulation_format);
    let mut result = String::new();
    let mut linebreak = false;
    let mut index = 0usize;

    while index < tokens.len() {
        match tokens[index] {
            Token::Word(word) => {
                let value: String;

                if word.keyword != Keyword::NoKeyword {
                    value = apply_case(format!("{}", word), &settings.keywords_case);

                    if settings.linebreak_before_keywords.contains(&value) && !linebreak {
                        result.push('\n');
                    }
                    result.push_str(&mut indentation.update(settings, &value));
                    linebreak = false;
                    if settings.linebreak_after_keywords.contains(&value) {
                        result.push('\n');
                        linebreak = true;
                    }
                } else {
                    value = format!("{}", word);
                    result.push_str(&mut indentation.update(settings, &value));
                    linebreak = false;
                    if settings.linebreak_after_words.contains(&value) {
                        result.push('\n');
                        linebreak = true;
                    }
                }
            }
            other_token => {
                let value = format!("{}", other_token);
                result.push_str(&mut indentation.update(settings, &value));
                linebreak = false;

                if settings.linebreak_after_words.contains(&value) {
                    result.push('\n');
                    linebreak = true;
                }
            }
        }

        index += 1;
    }

    println!("{:?}", result);
    Ok(result)
}

fn apply_case(word: String, case: &str) -> String {
    match case {
        "lower" | "lowercase" => word.to_lowercase(),
        "upper" | "uppercase" => word.to_uppercase(),
        _ => word,
    }
}
