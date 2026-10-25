use crate::atomic::{is_foreign_consonant, to_atomic};

use std::collections::HashSet;
use std::sync::OnceLock;

pub const TAG_PROPER_NAME: char = '\u{200B}';
pub const TAG_PLACE_NAME: char = '\u{200C}';
pub const TAG_DEITY_NAME: char = '\u{200D}';
pub const TAG_IMMUTABLE: char = '\u{200E}';

#[derive(Debug, Clone, Default)]
pub struct MetaWord {
    pub original: String,

    pub word: String,
    pub atomic: String,

    pub prefixed: String,
    pub postfixed: String,

    pub flic: bool,

    pub is_immutable: bool,
    pub is_proper_name: bool,
    pub is_place_name: bool,
    pub is_deity_name: bool,
}

const PREFIX_PUNCTUATION: &str =
    "\u{200B}\u{200C}\u{200D}\u{200E}\n\r.,—–‒/#!¡$%^&*;:=-_`~@+?¿(){}<>[]+\"“”«»‘’‛‹›…'";

const POSTFIX_PUNCTUATION: &str = "\n\r.,—–‒/#!¡$%^&*;:=-_`~@+?¿(){}<>[]+\"“”«»‘’‛‹›…'";

fn load_word_set(content: &'static str) -> HashSet<&'static str> {
    content
        .lines()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .collect()
}

fn given_names() -> &'static HashSet<&'static str> {
    static NAMES: OnceLock<HashSet<&'static str>> = OnceLock::new();
    NAMES.get_or_init(|| load_word_set(include_str!("data/given_names.txt")))
}

fn surnames() -> &'static HashSet<&'static str> {
    static NAMES: OnceLock<HashSet<&'static str>> = OnceLock::new();
    NAMES.get_or_init(|| load_word_set(include_str!("data/surnames.txt")))
}

fn deities() -> &'static HashSet<&'static str> {
    static DEITIES: OnceLock<HashSet<&'static str>> = OnceLock::new();
    DEITIES.get_or_init(|| load_word_set(include_str!("data/deities.txt")))
}

pub fn is_given_name(name: &str) -> bool {
    let name = name.to_lowercase();
    given_names().contains(name.as_str())
}

pub fn is_family_name(name: &str) -> bool {
    let name = name.to_lowercase();
    surnames().contains(name.as_str())
}

pub fn is_proper_name(name: &str) -> bool {
    let name = name.to_lowercase();
    given_names().contains(name.as_str()) || surnames().contains(name.as_str())
}

pub fn is_deity(name: &str) -> bool {
    deities().contains(name)
}

pub fn segregate_punctuation(input: &str) -> (String, String, String) {
    let chars: Vec<char> = input.chars().collect();
    let mut i = 0;

    let mut prefix = String::new();
    let mut word = String::new();
    let mut postfix = String::new();

    while i < chars.len() && PREFIX_PUNCTUATION.contains(chars[i]) {
        prefix.push(chars[i]);
        i += 1;
    }

    while i < chars.len() && !POSTFIX_PUNCTUATION.contains(chars[i]) {
        word.push(chars[i]);
        i += 1;
    }

    while i < chars.len() {
        postfix.push(chars[i]);
        i += 1;
    }

    (prefix, word, postfix)
}

pub fn has_locative_suffix(s: &str) -> bool {
    const SUFFIXES: &[&str] = &["ko", "λan", "tepek", "τinco", "singo", "apa", "apan", "kan"];

    let Some(suffix) = SUFFIXES.iter().find(|suffix| s.ends_with(**suffix)) else {
        return false;
    };

    let stem = &s[..s.len() - suffix.len()];

    for ch in stem.chars() {
        if is_foreign_consonant(ch) {
            return false;
        }
    }

    true
}

pub fn split_to_metawords(input: &str) -> Vec<MetaWord> {
    let mut result = Vec::new();

    for original in input.split_whitespace() {
        let (mut prefixed, word, postfixed) = segregate_punctuation(original);

        let is_immutable = prefixed.contains(TAG_IMMUTABLE);

        let mut proper_name = prefixed.contains(TAG_PROPER_NAME);
        let mut place_name = prefixed.contains(TAG_PLACE_NAME);
        let mut deity_name = prefixed.contains(TAG_DEITY_NAME);

        prefixed.retain(|c| {
            !matches!(
                c,
                TAG_IMMUTABLE | TAG_PROPER_NAME | TAG_PLACE_NAME | TAG_DEITY_NAME
            )
        });

        let atomic = to_atomic(&word.to_lowercase());

        let flic = word
            .chars()
            .next()
            .map(|c| c.is_uppercase())
            .unwrap_or(false);

        if !proper_name && !place_name && !deity_name {
            place_name = has_locative_suffix(&atomic);
            deity_name = is_deity(&atomic);
            proper_name = is_proper_name(&word);
        }

        result.push(MetaWord {
            original: original.to_string(),

            word,
            atomic,

            prefixed,
            postfixed,

            flic,

            is_immutable,
            is_proper_name: proper_name,
            is_place_name: place_name,
            is_deity_name: deity_name,
        });
    }

    result
}
