//! F→M (phonetic-to-morphophonemic) restoration rules.
//!
//! These transformations are used when converting text from
//! phonetic orthographies (such as SEP-style spellings) back
//! toward morphophonemic orthographies.
//!
//! The process is inherently heuristic and cannot perfectly
//! reconstruct the original spelling because phonetic spellings
//! often omit distinctions present in morphophonemic systems.

use once_cell::sync::Lazy;
use regex::Regex;
use std::sync::OnceLock;

fn en_endings() -> &'static [&'static str] {
    static ENDINGS: OnceLock<Vec<&'static str>> = OnceLock::new();
    ENDINGS.get_or_init(|| {
        include_str!("data/en_endings.txt")
            .lines()
            .map(str::trim)
            .filter(|s| !s.is_empty() && !s.starts_with('#'))
            .collect()
    })
}

/// Returns true if `word` ends with a suffix that historically
/// surfaces as `-en` in morphophonemic orthographies.
pub fn ends_in_en(word: &str) -> bool {
    en_endings().iter().any(|suffix| word.ends_with(suffix))
}

// Extracted prefixes for HK2WK regex
// Note: If prefixes ever become dynamic, remember to use regex::escape.
static HK2WK_PREFIXES: &[&str] = &[
    "ala",
    "kana",
    "ka",
    "se",
    "ςipa",
    "ςi",
    "ςo",
    "sia",
    "si",
    "ko",
    "κahκa",
    "κite",
    "e",
    "wapa",
    "itonalpote",
    "li",
    "lo",
    "ma",
    "maxihxiλa",
    "mela",
    "me",
    "mole",
    "ne",
    "ni",
    "paλa",
    "pe",
    "pina",
    "piτa",
    "pohye",
    "poke",
    "po",
    "ki",
    "tila",
    "λaxλa",
    "λiκeti",
    "τa",
    "τi",
    "xa",
    "xiλa",
    "xi",
    "ya",
    "yo",
    "sa",
    "soλa",
    "so",
];

static HK2WK: Lazy<Regex> = Lazy::new(|| {
    let pattern = format!("({})hk(i|eh)$", HK2WK_PREFIXES.join("|"));
    Regex::new(&pattern).unwrap()
});

/// Restores syllable-final /w/ that was realized as [h]
/// in certain phonetic spellings.
///
/// Examples:
///
/// - pehki  -> pewki
/// - pehkeh -> pewkeh
/// - kohki  -> kowki
pub fn hk2wk(word: &str) -> String {
    HK2WK.replace(word, "${1}wk${2}").into_owned()
}

static HCONS2KCONS: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"([nt]i)h([mnptkκτλςsxhlwyñβdgfrρbv])([aeio])").unwrap());

/// Converts certain consonantal [h] realizations back to
/// morphophonemic /k/ based on known verbal patterns.
pub fn h_cons_to_k_cons(word: &str) -> String {
    HCONS2KCONS.replace(word, "${1}k${2}${3}").into_owned()
}

/// Restores word-final /w/ for a small set of common words
/// that are frequently written with final [h] in phonetic
/// orthographies.
pub fn h2w(word: &str) -> String {
    match word {
        "peτikih" => "peτikiw".to_string(),
        "tonatih" => "tonatiw".to_string(),
        "koneh" => "konew".to_string(),
        _ => word.to_string(),
    }
}

// Exception lists for suffix transformations
const TZIH_EXCEPTIONS: &[&str] = &["τahτih", "weτih"];
const SECIH_EXCEPTIONS: &[&str] = &["leςih", "peςih"];
const LIH_EXCEPTIONS: &[&str] = &["kikilih"];

// Explicit lexical corrections derived from NOCE.
//
// Ordering is significant because later replacements operate
// on the results of earlier substitutions.
const NOCE_REPLACEMENTS: &[(&str, &str)] = &[
    ("iniwah", "inihwan"),
    ("iniwantih", "inihwantin"),
    ("inmowah", "inmohwan"),
    ("inmowantih", "inmohwantin"),
    ("towah", "tohwan"),
    ("towantih", "tohwantin"),
];

fn apply_suffix_unless(w: &mut String, suffix: &str, replacement: &str, exceptions: &[&str]) {
    if !exceptions.iter().any(|&ex| w.ends_with(ex))
        && let Some(stem) = w.strip_suffix(suffix)
    {
        *w = format!("{stem}{replacement}");
    }
}

/// Restores word-final /n/ from phonetic spellings that
/// surface final /n/ as [h], using a combination of suffix
/// rules, exception lists, and lexical patterns.
///
/// This is an intentionally conservative heuristic intended
/// to reduce false-positive conversions.
pub fn h2n(word: &str) -> String {
    let mut w = word.to_string();

    apply_suffix_unless(&mut w, "τih", "τin", TZIH_EXCEPTIONS);
    apply_suffix_unless(&mut w, "ςih", "ςin", SECIH_EXCEPTIONS);
    apply_suffix_unless(&mut w, "lih", "lin", LIH_EXCEPTIONS);

    // Explicit exceptions from NOCE
    for (from, to) in NOCE_REPLACEMENTS {
        w = w.replace(from, to);
    }

    // en_endings.js
    if ends_in_en(&w) && w.ends_with('h') {
        w.pop();
        w.push('n');
    }

    w
}

/// Applies the complete phonetic-to-morphophonemic restoration
/// pipeline.
///
/// Transformations are applied in the same order as the original
/// Nahuatl Tools implementation.
pub fn apply_f2m(word: &str) -> String {
    let word = hk2wk(word);
    let word = h_cons_to_k_cons(&word);
    let word = h2w(&word);
    h2n(&word)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hk2wk_pehki() {
        assert_eq!(hk2wk("pehki"), "pewki");
    }

    #[test]
    fn hk2wk_kohkeh() {
        assert_eq!(hk2wk("kohkeh"), "kowkeh");
    }

    #[test]
    fn hcons2kcons() {
        assert_eq!(h_cons_to_k_cons("nihneki"), "nikneki");
    }

    #[test]
    fn h2w_tonatih() {
        assert_eq!(h2w("tonatih"), "tonatiw");
    }

    #[test]
    fn h2n_tzih() {
        assert_eq!(h2n("τih"), "τin");
    }

    #[test]
    fn h2n_michih() {
        assert_eq!(h2n("miςih"), "miςin");
    }

    #[test]
    fn h2n_totolih() {
        assert_eq!(h2n("totolih"), "totolin");
    }
}
