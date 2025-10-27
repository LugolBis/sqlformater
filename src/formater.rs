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
        Indentation(0, 0, String::from(value))
    }

    fn update(&mut self, settings: &Settings, word: &str, linebreak: bool) -> String {
        let (start, end, reset) = (
            &settings.indentation_start,
            &settings.indentation_end,
            &settings.indentation_reset,
        );

        match (
            start.contains(word),
            end.contains(word),
            reset.contains(word),
        ) {
            (true, false, true) | (true, true, true) => {
                self.0 = 0;
                self.1 += 1;
            }
            (true, true, false) => {
                self.0 = self.0.saturating_sub(1);
                self.1 += 1;
            }
            (true, _, _) => {
                self.1 += 1;
            }
            (false, true, true) | (false, false, true) => {
                self.0 = 0;
                self.1 = 0;
            }
            (false, true, false) => {
                self.0 = self.0.saturating_sub(1);
                self.1 = self.1.saturating_sub(1);
            }
            (false, false, false) => {}
        }
        self.get_string(word, linebreak)
    }

    fn get_string(&mut self, word: &str, linebreak: bool) -> String {
        let indentation: String;
        if linebreak {
            indentation = self.2.repeat(self.0);
        } else {
            indentation = String::new();
        }
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
    let mut buffer = String::new();
    let mut result = String::new();
    let mut index = 0usize;

    while index < tokens.len() {
        match tokens[index] {
            Token::Word(word) => {
                let value: String;

                if word.keyword != Keyword::NoKeyword {
                    value = apply_case(format!("{}", word), &settings.keywords_case);

                    if settings.linebreak_before_keywords.contains(&value)
                        && !result.ends_with("\n")
                    {
                        buffer.push('\n');
                    }
                    buffer.push_str(&mut indentation.update(
                        settings,
                        &value,
                        result.ends_with("\n"),
                    ));
                    if settings.linebreak_after_keywords.contains(&value) && !result.ends_with("\n")
                    {
                        buffer.push('\n');
                    }
                } else {
                    value = format!("{}", word);
                    buffer.push_str(&mut indentation.update(
                        settings,
                        &value,
                        result.ends_with("\n"),
                    ));

                    if settings.linebreak_after_words.contains(&value) && !result.ends_with("\n") {
                        buffer.push('\n');
                    }
                }
            }
            other_token => {
                let value = format!("{}", other_token);
                buffer.push_str(&mut indentation.update(settings, &value, result.ends_with("\n")));

                if settings.linebreak_after_words.contains(&value) && !result.ends_with("\n") {
                    buffer.push('\n');
                }
            }
        }

        result.push_str(&buffer);
        buffer.clear();
        index += 1;
    }

    Ok(result)
}

fn apply_case(word: String, case: &str) -> String {
    match case {
        "lower" | "lowercase" => word.to_lowercase(),
        "upper" | "uppercase" => word.to_uppercase(),
        _ => word,
    }
}
