//! Morphophonemic (M) ↔ phonetic/allophonic (F) transformations.
//!
//! This module implements sound changes used when deriving
//! phonetic spellings and pronunciations from morphophonemic
//! ATOMIC forms.
//!
//! The transformations model common Nahuatl allophones such as:
//!
//! - /n/ -> [h] word-finally
//! - /w/ -> [h] in syllable codas
//! - /kʷ/ -> [h] before /k/
//! - /k/ -> [h] before certain consonants
//! - /ll/ -> [hl] in specific environments
//!
//! These rules are also used to construct phonetic IPA output
//! and search forms.

use crate::atomic::{is_atomic_consonant, is_atomic_vowel};
use crate::orthography::atomic_to_ipa;
use once_cell::sync::Lazy;
use regex::Regex;

// Shared consonant inventory for regex character classes
const CONSONANTS: &str = "mnptkκτλςsxhlwyñβdgfrρbv";
const CONSONANTS_WITH_SPACE: &str = "mnptkκτλςsxhlwyñβdgfrρbv ";

static LL2HL: Lazy<Regex> = Lazy::new(|| Regex::new(r"([i])l(l[ai])").unwrap());

/// Applies the /ll/ → [hl] allophone rule in environments
/// such as illia → ihlia.
pub fn ll2hl(word: &str) -> String {
    LL2HL.replace(word, "${1}h${2}").into_owned()
}

static N2H_FINAL: Lazy<Regex> = Lazy::new(|| Regex::new(r"n$").unwrap());

/// Applies the common word-final /n/ → [h] allophone rule.
pub fn n2h(word: &str) -> String {
    N2H_FINAL.replace(word, "h").into_owned()
}

static W2H: Lazy<Regex> =
    Lazy::new(|| Regex::new(&format!(r"([aeio])w([{CONSONANTS_WITH_SPACE}])")).unwrap());

static W2H_FINAL: Lazy<Regex> = Lazy::new(|| Regex::new(r"([aeio])w$").unwrap());

/// Applies the syllable-coda /w/ → [h] allophone rule.
pub fn w2h(word: &str) -> String {
    let word = W2H.replace_all(word, "${1}h${2}");
    let word = W2H_FINAL.replace(&word, "${1}h");
    word.into_owned()
}

static KW2H: Lazy<Regex> = Lazy::new(|| Regex::new(r"([aeio])κ(k)").unwrap());

static KW2K: Lazy<Regex> =
    Lazy::new(|| Regex::new(&format!(r"([aeio])κ([{CONSONANTS_WITH_SPACE}])")).unwrap());

static KW2K_FINAL: Lazy<Regex> = Lazy::new(|| Regex::new(r"([aeio])κ$").unwrap());

/// Converts coda /kʷ/ to its phonetic realizations:
/// [h] before /k/, [k] elsewhere.
pub fn kw2k_or_h(word: &str) -> String {
    let word = KW2H.replace_all(word, "${1}h${2}");
    let word = KW2K.replace_all(&word, "${1}k${2}");
    let word = KW2K_FINAL.replace(&word, "${1}k");

    word.into_owned()
}

static KCONS2HCONS: Lazy<Regex> =
    Lazy::new(|| Regex::new(&format!(r"([aeio])k([{CONSONANTS}])([aeio])")).unwrap());

/// Applies the /k/ → [h] coda-before-consonant allophone rule.
pub fn k_cons_to_h_cons(word: &str) -> String {
    KCONS2HCONS.replace(word, "${1}h${2}${3}").into_owned()
}

/// Applies the complete morphophonemic-to-phonetic (M→F)
/// transformation pipeline.
///
/// The resulting form reflects common allophonic spellings
/// used by phonetic orthographies such as SEP and forms the
/// basis for phonetic IPA generation.
pub fn apply_m2f(word: &str) -> String {
    let word = ll2hl(word);
    let word = n2h(&word);
    let word = w2h(&word);
    let word = kw2k_or_h(&word);

    k_cons_to_h_cons(&word)
}

/// Removes adjacent identical consonants.
///
/// This is primarily used when constructing search forms and
/// phonetic orthographies that do not preserve geminate
/// consonants.
pub fn degeminate(word: &str) -> String {
    let mut out = String::new();
    let chars: Vec<char> = word.chars().collect();

    let mut i = 0;
    while i < chars.len() {
        out.push(chars[i]);

        if i + 1 < chars.len() && chars[i] == chars[i + 1] && is_atomic_consonant(chars[i]) {
            i += 2;
        } else {
            i += 1;
        }
    }

    out
}

/// Inserts syllable boundaries using a simple ATOMIC
/// syllabification heuristic.
pub fn syllabify(word: &str) -> String {
    let mut out = String::new();
    let chars: Vec<char> = word.chars().collect();

    let mut i = 0;

    while i < chars.len() {
        let current = chars[i];

        let next = chars.get(i + 1).copied();
        let next2 = chars.get(i + 2).copied();

        let needs_break = (is_atomic_vowel(current)
            && next.is_some_and(is_atomic_consonant)
            && next2.is_some_and(is_atomic_vowel))
            || (is_atomic_consonant(current) && next.is_some_and(is_atomic_consonant))
            || (is_atomic_vowel(current) && next.is_some_and(is_atomic_vowel));

        out.push(current);

        if needs_break {
            out.push('·');
        }

        i += 1;
    }

    out
}

/// Marks penultimate stress using the IPA primary-stress
/// symbol (ˈ).
pub fn mark_penultimate_stress(s: &str) -> String {
    let mut positions = Vec::new();

    for (idx, ch) in s.char_indices() {
        if ch == '·' {
            positions.push(idx);
        }
    }

    if positions.len() >= 2 {
        let pos = positions[positions.len() - 2];

        let mut result = String::new();
        result.push_str(&s[..pos]);
        result.push('ˈ');
        result.push_str(&s[pos + '·'.len_utf8()..]);

        result
    } else {
        format!("ˈ{s}")
    }
}

/// Lexical exceptions to the default penultimate-stress rule.
const STRESS_EXCEPTIONS: &[(&str, &str)] = &[("ˈaʃ·t͡ɬen", "aʃˈt͡ɬen"), ("ˈjon.se", "jonˈse")];

/// Applies lexical corrections for words whose stress
/// placement differs from the default penultimate pattern.
pub fn fix_stress_exceptions(iph: &str) -> String {
    let mut res = iph.to_string();
    for (from, to) in STRESS_EXCEPTIONS {
        res = res.replace(from, to);
    }
    res
}

/// Produces a phonetic IPA transcription.
///
/// Unlike `atomic_to_ipa()`, this function first applies
/// allophonic processing, syllabification, stress assignment,
/// and stress-correction rules before generating IPA output.
pub fn atomic_to_ipa_phonetic(word: &str) -> String {
    let degem = degeminate(word);
    let syllables = syllabify(&degem);
    let stressed = mark_penultimate_stress(&syllables);

    let ipa = atomic_to_ipa(&stressed);

    fix_stress_exceptions(&ipa)
}

/// Produces a simplified search form.
///
/// Search forms are derived by applying phonetic allophonic
/// rules and removing geminated consonants so that common
/// spelling variations remain searchable.
pub fn search_form(word: &str) -> String {
    degeminate(&apply_m2f(word))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ll2hl_illia() {
        assert_eq!(ll2hl("illia"), "ihlia");
    }

    #[test]
    fn n2h_tzin() {
        assert_eq!(n2h("τin"), "τih");
    }

    #[test]
    fn w2h_tonatiw() {
        assert_eq!(w2h("tonatiw"), "tonatih");
    }

    #[test]
    fn kw2k_final() {
        assert_eq!(kw2k_or_h("aκ"), "ak");
    }

    #[test]
    fn kw2h_before_k() {
        assert_eq!(kw2k_or_h("aκka"), "ahka");
    }

    #[test]
    fn kcons2hcons() {
        assert_eq!(k_cons_to_h_cons("nikneki"), "nihneki");
    }

    #[test]
    fn apply_m2f_nihneki() {
        assert_eq!(apply_m2f("nikneki"), "nihneki");
    }

    #[test]
    fn apply_m2f_tonatiw() {
        assert_eq!(apply_m2f("tonatiw"), "tonatih");
    }

    #[test]
    fn apply_m2f_tzin() {
        assert_eq!(apply_m2f("τin"), "τih");
    }

    #[test]
    fn degeminate_kalli() {
        assert_eq!(degeminate("kalli"), "kali");
    }

    #[test]
    fn degeminate_itta() {
        assert_eq!(degeminate("itta"), "ita");
    }

    #[test]
    fn degeminate_non_geminate() {
        assert_eq!(degeminate("kali"), "kali");
    }

    #[test]
    fn syllabify_vcv() {
        assert_eq!(syllabify("kali"), "ka·li");
    }

    #[test]
    fn syllabify_cc() {
        assert_eq!(syllabify("kalli"), "kal·li");
    }

    #[test]
    fn syllabify_vv() {
        assert_eq!(syllabify("toa"), "to·a");
    }

    #[test]
    fn stress_two_syllables() {
        assert_eq!(mark_penultimate_stress("ka·li"), "ˈka·li");
    }

    #[test]
    fn stress_three_syllables() {
        assert_eq!(mark_penultimate_stress("ta·ka·ti"), "taˈka·ti");
    }

    #[test]
    fn stress_exception_ashtlen() {
        assert_eq!(fix_stress_exceptions("ˈaʃ·t͡ɬen"), "aʃˈt͡ɬen");
    }

    #[test]
    fn stress_exception_yonse() {
        assert_eq!(fix_stress_exceptions("ˈjon.se"), "jonˈse");
    }

    #[test]
    fn ipa_phonetic_lambda() {
        assert_eq!(atomic_to_ipa_phonetic("λ"), "ˈt͡ɬ");
    }

    #[test]
    fn ipa_phonetic_kalli() {
        assert_eq!(atomic_to_ipa_phonetic("kalli"), "ˈka·li");
    }

    #[test]
    fn search_form_nihneki() {
        assert_eq!(search_form("nikneki"), "nihneki");
    }

    #[test]
    fn search_form_kalli() {
        assert_eq!(search_form("kalli"), "kali");
    }
}
