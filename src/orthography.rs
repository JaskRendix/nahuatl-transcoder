//! Conversion from ATOMIC orthography to external orthographies
//! and phonetic representations.
//!
//! This module provides exporters for:
//!
//! - SEP orthography
//! - Hasler orthography
//! - ACK orthography
//! - IPA transcription
//!
//! Most conversions are simple character mappings, while
//! ACK requires context-sensitive spelling rules.

use crate::atomic::{is_atomic_letter, is_atomic_vowel};

fn map_sep(ch: char) -> Option<&'static str> {
    Some(match ch {
        'a' => "a",
        'e' => "e",
        'i' => "i",
        'o' => "o",
        'u' => "u",

        'ā' => "ā",
        'ē' => "ē",
        'ī' => "ī",
        'ō' => "ō",
        'ū' => "ū",

        'm' => "m",
        'n' => "n",
        'p' => "p",
        't' => "t",

        'k' => "k",
        'κ' => "ku",

        'τ' => "ts",
        'λ' => "tl",
        'ς' => "ch",

        's' => "s",
        'l' => "l",
        'x' => "x",

        'ʔ' => "j",
        'h' => "j",

        'y' => "y",
        'w' => "u",

        'ñ' => "ñ",
        'β' => "b",
        'b' => "b",
        'v' => "v",

        'd' => "d",
        'g' => "g",
        'f' => "f",

        'r' => "r",
        'ρ' => "rr",

        _ => return None,
    })
}

fn map_hasler(ch: char) -> Option<&'static str> {
    Some(match ch {
        'a' => "a",
        'e' => "e",
        'i' => "i",
        'o' => "o",
        'u' => "u",

        'ā' => "ā",
        'ē' => "ē",
        'ī' => "ī",
        'ō' => "ō",
        'ū' => "ū",

        'm' => "m",
        'n' => "n",
        'p' => "p",
        't' => "t",

        'k' => "k",
        'κ' => "ku",

        'τ' => "tz",
        'λ' => "tl",
        'ς' => "ch",

        's' => "s",
        'l' => "l",
        'x' => "x",

        'ʔ' => "h",
        'h' => "h",

        'y' => "y",
        'w' => "w",

        'ñ' => "ñ",
        'β' => "b",
        'b' => "b",
        'v' => "v",

        'd' => "d",
        'g' => "g",
        'f' => "f",

        'r' => "r",
        'ρ' => "rr",

        _ => return None,
    })
}

fn map_ipa(ch: char) -> Option<&'static str> {
    Some(match ch {
        'a' => "a",
        'e' => "e",
        'i' => "i",
        'o' => "o",
        'u' => "u",

        'ā' => "aː",
        'ē' => "eː",
        'ī' => "iː",
        'ō' => "oː",
        'ū' => "uː",

        'm' => "m",
        'n' => "n",
        'p' => "p",
        't' => "t",

        'k' => "k",
        'κ' => "kʷ",

        'τ' => "t͡s",
        'λ' => "t͡ɬ",
        'ς' => "t͡ʃ",

        's' => "s",
        'l' => "l",

        'x' => "ʃ",

        'ʔ' => "ʔ",
        'h' => "h",

        'y' => "j",
        'w' => "w",

        'ñ' => "ɲ",

        'β' => "β",
        'b' => "b",
        'v' => "v",

        'd' => "d",
        'g' => "g",
        'f' => "f",

        'r' => "ɾ",
        'ρ' => "r",

        _ => return None,
    })
}

/// Applies a character-mapping function to every character
/// in the input string.
///
/// Characters without a mapping are copied unchanged.
fn transliterate<F>(input: &str, map_fn: F) -> String
where
    F: Fn(char) -> Option<&'static str>,
{
    let mut out = String::new();
    for ch in input.chars() {
        match map_fn(ch) {
            Some(m) => out.push_str(m),
            None => out.push(ch),
        }
    }
    out
}

/// Converts ATOMIC text to SEP orthography.
pub fn atomic_to_sep(input: &str) -> String {
    transliterate(input, map_sep)
}

/// Converts ATOMIC text to Hasler orthography.
pub fn atomic_to_hasler(input: &str) -> String {
    transliterate(input, map_hasler)
}

/// Converts ATOMIC text to IPA.
pub fn atomic_to_ipa(input: &str) -> String {
    transliterate(input, map_ipa)
}

/// Converts ATOMIC text to ACK orthography.
///
/// Unlike the other exporters, ACK orthography requires
/// context-sensitive spelling rules such as:
///
/// - w → hu / uh
/// - κ → cu / uc
/// - k → qu before e/i
/// - k → c elsewhere
/// - s → c before e/i
/// - s → z elsewhere
pub fn atomic_to_ack(input: &str) -> String {
    let chars: Vec<char> = input.chars().collect();
    let mut out = String::new();

    for i in 0..chars.len() {
        let current = chars[i];
        let prev = if i > 0 { chars[i - 1] } else { ' ' };
        let next = if i + 1 < chars.len() {
            chars[i + 1]
        } else {
            ' '
        };

        // w -> hu / uh
        if current == 'w' && is_atomic_vowel(prev) && !is_atomic_vowel(next) {
            out.push_str("uh");
            continue;
        }

        // κ -> cu / uc
        if current == 'κ' && is_atomic_vowel(prev) && !is_atomic_vowel(next) {
            out.push_str("uc");
            continue;
        }

        // k before e/i
        if current == 'k' && matches!(next, 'e' | 'i' | 'ē' | 'ī') {
            out.push_str("qu");
            continue;
        }

        // s before e/i
        if current == 's' && matches!(next, 'e' | 'i' | 'ē' | 'ī') {
            out.push('c');
            continue;
        }

        // k before a/o
        if current == 'k' {
            out.push('c');
            continue;
        }

        // s before a/o
        if current == 's' {
            out.push('z');
            continue;
        }

        if is_atomic_letter(current) {
            match current {
                'a' => out.push('a'),
                'e' => out.push('e'),
                'i' => out.push('i'),
                'o' => out.push('o'),
                'u' => out.push('u'),

                'ā' => out.push('ā'),
                'ē' => out.push('ē'),
                'ī' => out.push('ī'),
                'ō' => out.push('ō'),
                'ū' => out.push('ū'),

                'm' => out.push('m'),
                'n' => out.push('n'),
                'p' => out.push('p'),
                't' => out.push('t'),

                'κ' => out.push_str("cu"),

                'τ' => out.push_str("tz"),
                'λ' => out.push_str("tl"),
                'ς' => out.push_str("ch"),

                'x' => out.push('x'),

                'ʔ' => out.push('h'),
                'h' => out.push('h'),

                'y' => out.push('y'),
                'w' => out.push_str("hu"),

                'ñ' => out.push('ñ'),

                'β' => out.push('b'),

                'b' => out.push('b'),
                'v' => out.push('v'),

                'd' => out.push('d'),
                'g' => out.push('g'),
                'f' => out.push('f'),

                'r' => out.push('r'),
                'ρ' => out.push_str("rr"),

                _ => out.push(current),
            }
        } else {
            out.push(current);
        }
    }

    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn atomic_ack() {
        assert_eq!(atomic_to_ack("κawλi"), "cuauhtli");
    }

    #[test]
    fn atomic_sep() {
        assert_eq!(atomic_to_sep("κawλi"), "kuautli");
    }

    #[test]
    fn atomic_hasler() {
        assert_eq!(atomic_to_hasler("κawλi"), "kuawtli");
    }

    #[test]
    fn atomic_ipa() {
        assert_eq!(atomic_to_ipa("λ"), "t͡ɬ");
    }
}
