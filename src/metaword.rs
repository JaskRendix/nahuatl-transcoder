//! MetaWord parsing and named-entity detection.
//!
//! This module splits input text into `MetaWord` records,
//! preserving surrounding punctuation while normalizing words
//! into the internal ATOMIC orthography.
//!
//! During parsing, words may be classified as:
//!
//! - proper names
//! - place names
//! - deity names
//! - immutable words
//!
//! Classification can be driven by explicit tags or inferred
//! from lexical data and orthographic heuristics.

use crate::atomic::{is_foreign_consonant, to_atomic};

use std::collections::HashSet;
use std::sync::OnceLock;

/// Explicit tag marking a proper name.
pub const TAG_PROPER_NAME: char = '\u{200B}';

/// Explicit tag marking a place name.
pub const TAG_PLACE_NAME: char = '\u{200C}';

/// Explicit tag marking a deity name.
pub const TAG_DEITY_NAME: char = '\u{200D}';

/// Explicit tag marking an immutable word.
pub const TAG_IMMUTABLE: char = '\u{200E}';

/// A parsed word together with normalization and
/// classification metadata.
///
/// `MetaWord` preserves the original spelling while also
/// storing a normalized ATOMIC representation used by
/// orthographic conversion pipelines.
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

/// Loads a newline-delimited word list into a hash set.
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

/// Returns true if `name` is listed as a given name.
pub fn is_given_name(name: &str) -> bool {
    let name = name.to_lowercase();
    given_names().contains(name.as_str())
}

/// Returns true if `name` is listed as a family name.
pub fn is_family_name(name: &str) -> bool {
    let name = name.to_lowercase();
    surnames().contains(name.as_str())
}

/// Returns true if `name` is recognized as a proper name.
pub fn is_proper_name(name: &str) -> bool {
    let name = name.to_lowercase();
    given_names().contains(name.as_str()) || surnames().contains(name.as_str())
}

/// Returns true if `name` matches a known Nahuatl deity.
pub fn is_deity(name: &str) -> bool {
    deities().contains(name)
}

/// Separates leading punctuation, the core word,
/// and trailing punctuation.
///
/// Returns `(prefix, word, postfix)`.
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

/// Heuristically identifies place names by testing for
/// common Nahuatl locative suffixes.
///
/// Words containing foreign consonants are excluded from
/// automatic place-name detection.
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

/// Splits text into `MetaWord` records.
///
/// Each word is:
///
/// - separated from surrounding punctuation
/// - normalized into ATOMIC orthography
/// - checked for capitalization
/// - classified as a proper name, place name, deity name,
///   or immutable term
///
/// Explicit tags take precedence over heuristic detection.
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
