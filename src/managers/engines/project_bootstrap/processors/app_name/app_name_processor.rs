use crate::managers::exceptions::application_base_exception::{ApplicationBaseException, ApplicationResult};
use crate::managers::exceptions::contextual_data::ContextualData;
use crate::managers::exceptions::project_bootstrap_engine_exception::ProjectBootstrapEngineException;
use crate::managers::models::app_name::AppName;
use crate::managers::models::name_replacement::NameReplacement;

const MAX_APP_NAME_CHARS: usize = 60;

// Belongs to the ProjectBootstrapEngine: the forms of a new app's name, and the template's names replaced with them.
pub struct AppNameProcessor;

impl AppNameProcessor {
    /// The forms of the name the person gave ("Orders App", "orders-app", "OrdersApp" are all orders-app): its words,
    /// split at anything that isn't a letter or a digit and where a lowercase letter or a digit meets a capital. Every
    /// form must be a name in any stack (a package, a crate, a type), so the name starts with a letter and its words
    /// hold ASCII letters and digits only.
    pub fn to_app_name(&self, name: &str) -> ApplicationResult<AppName> {
        let words: Vec<String> = AppNameProcessor::split_words(name);
        let starts_with_letter: bool = words
            .first()
            .and_then(|word: &String| word.chars().next())
            .is_some_and(|character: char| character.is_ascii_alphabetic());
        let is_ascii: bool = name.is_ascii();
        // Its kebab-case form names a folder, a package and a database, each with a limit of its own.
        let is_short_enough: bool = name.len() <= MAX_APP_NAME_CHARS;
        if !starts_with_letter || !is_ascii || !is_short_enough {
            return Err(ApplicationBaseException::new(
                ProjectBootstrapEngineException::InvalidAppName,
                "The app's name must start with a letter, hold only letters, digits, spaces, hyphens and \
                 underscores (e.g. orders-app), and be at most 60 characters long",
                ContextualData::new().with("app_name", name),
            ));
        }

        let capitalized: Vec<String> = words
            .iter()
            .map(|word: &String| AppNameProcessor::capitalize(word))
            .collect();
        // The first word as it is, the others capitalized: the name of a variable or a field holding the app's facade.
        let camel: String = words
            .iter()
            .enumerate()
            .map(|(index, word): (usize, &String)| {
                if index == 0 {
                    word.clone()
                } else {
                    AppNameProcessor::capitalize(word)
                }
            })
            .collect();
        Ok(AppName {
            kebab: words.join("-"),
            snake: words.join("_"),
            pascal: capitalized.concat(),
            camel,
            upper: words.join("_").to_ascii_uppercase(),
            title: capitalized.join(" "),
        })
    }

    /// The text with every template name replaced, in one pass, the longest name first where several start at the
    /// same place; what a replacement puts in is never replaced again.
    pub fn replace_names(&self, text: &str, name_replacements: &[NameReplacement], app_name: &AppName) -> String {
        let mut longest_first: Vec<(&str, String)> = name_replacements
            .iter()
            .filter(|name_replacement: &&NameReplacement| !name_replacement.name.is_empty())
            .map(|name_replacement: &NameReplacement| {
                (
                    name_replacement.name.as_str(),
                    AppNameProcessor::fill(&name_replacement.replacement, app_name),
                )
            })
            .collect();
        longest_first.sort_by_key(|(name, _): &(&str, String)| std::cmp::Reverse(name.len()));

        let mut replaced: String = String::with_capacity(text.len());
        let mut rest: &str = text;
        'scan: while let Some(character) = rest.chars().next() {
            for (name, replacement) in &longest_first {
                if let Some(after) = rest.strip_prefix(name) {
                    replaced.push_str(replacement);
                    rest = after;
                    continue 'scan;
                }
            }
            replaced.push(character);
            rest = &rest[character.len_utf8()..];
        }
        replaced
    }

    // A replacement's placeholders filled with the forms of the app's name.
    fn fill(replacement: &str, app_name: &AppName) -> String {
        replacement
            .replace("{app_name_kebab}", &app_name.kebab)
            .replace("{app_name_snake}", &app_name.snake)
            .replace("{app_name_pascal}", &app_name.pascal)
            .replace("{app_name_camel}", &app_name.camel)
            .replace("{app_name_upper}", &app_name.upper)
            .replace("{app_name_title}", &app_name.title)
    }

    // Lowercase words: a new word at anything that isn't an ASCII letter or digit, and where a lowercase letter or a
    // digit is followed by a capital ("OrdersApp" is orders, app).
    fn split_words(name: &str) -> Vec<String> {
        let mut words: Vec<String> = Vec::new();
        let mut word: String = String::new();
        let mut previous: Option<char> = None;
        for character in name.chars() {
            if !character.is_ascii_alphanumeric() {
                if !word.is_empty() {
                    words.push(std::mem::take(&mut word));
                }
                previous = None;
                continue;
            }
            let starts_word: bool = character.is_ascii_uppercase()
                && previous.is_some_and(|previous: char| previous.is_ascii_lowercase() || previous.is_ascii_digit());
            if starts_word && !word.is_empty() {
                words.push(std::mem::take(&mut word));
            }
            word.push(character.to_ascii_lowercase());
            previous = Some(character);
        }
        if !word.is_empty() {
            words.push(word);
        }
        words
    }

    fn capitalize(word: &str) -> String {
        let mut characters: std::str::Chars<'_> = word.chars();
        match characters.next() {
            Some(first) => first.to_ascii_uppercase().to_string() + characters.as_str(),
            None => String::new(),
        }
    }
}
