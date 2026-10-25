pub mod abugida;
pub mod allophones;
pub mod atomic;
pub mod constants;
pub mod geminate;
pub mod metaword;
pub mod orthography;
pub mod phonetics;
pub mod ranking;
pub mod reconstruction;
pub mod structure;
pub mod trager;

use abugida::atomic_to_abugida;
use allophones::*;
use atomic::to_atomic;
use metaword::split_to_metawords;
use orthography::{atomic_to_ack, atomic_to_hasler, atomic_to_ipa, atomic_to_sep};
use phonetics::*;
use structure::structure;

#[derive(Debug, Clone, Default)]
pub struct ConversionResult {
    pub trager: String,
    pub ack: String,
    pub sep: String,
    pub hasler: String,
    pub atom: String,
    pub allo: String,
    pub ipa: String,
    pub iph: String,
    pub search: String,
}

pub fn convert_all(input: &str) -> ConversionResult {
    let atomic = to_atomic(input);

    let allo = apply_m2f(&atomic);

    ConversionResult {
        trager: structure(&atomic_to_abugida(&atomic)),
        ack: atomic_to_ack(&atomic),
        sep: atomic_to_sep(&allo),
        hasler: atomic_to_hasler(&allo),
        atom: atomic.clone(),
        allo: allo.clone(),
        ipa: atomic_to_ipa(&atomic),
        iph: atomic_to_ipa_phonetic(&allo),
        search: search_form(&atomic),
    }
}

pub fn convert(input: &str) -> String {
    let atomic = to_atomic(input);

    let raw_abugida = atomic_to_abugida(&atomic);

    structure(&raw_abugida)
}

pub fn convert_text(input: &str) -> String {
    let words = split_to_metawords(input);

    let mut out = String::new();

    for mw in words {
        if mw.is_immutable {
            out.push_str(&mw.prefixed);
            out.push_str(&mw.word);
            out.push_str(&mw.postfixed);
            out.push(' ');
            continue;
        }

        let abugida = structure(&atomic_to_abugida(&mw.atomic));

        out.push_str(&mw.prefixed);

        if mw.is_deity_name {
            out.push(constants::nab::PREFIX_DEITY);
        } else if mw.is_place_name {
            out.push(constants::nab::PREFIX_PLACE);
        } else if mw.is_proper_name {
            out.push(constants::nab::PREFIX_NAME);
        }

        out.push_str(&abugida);
        out.push_str(&mw.postfixed);
        out.push(' ');
    }

    out.trim_end().to_string()
}

pub fn apply_f2m(word: &str) -> String {
    let word = hk2wk(word);
    let word = h_cons_to_k_cons(&word);
    let word = h2w(&word);
    h2n(&word)
}

pub fn apply_m2f(word: &str) -> String {
    let word = ll2hl(word);
    let word = n2h(&word);
    let word = w2h(&word);
    let word = kw2k_or_h(&word);

    k_cons_to_h_cons(&word)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn convert_text_empty() {
        assert_eq!(convert_text(""), "");
    }

    #[test]
    fn convert_text_multiple_words() {
        let result = convert_text("kalli kuawitl");
        assert!(result.contains(' '));
    }

    #[test]
    fn convert_text_proper_name_prefix() {
        let result = convert_text("Aaron");

        assert!(result.starts_with(constants::nab::PREFIX_NAME));
    }

    #[test]
    fn convert_text_place_prefix() {
        let result = convert_text("Mexico");

        assert!(result.starts_with(constants::nab::PREFIX_PLACE));
    }

    #[test]
    fn convert_text_deity_prefix() {
        let result = convert_text("λalok");

        assert!(result.starts_with(constants::nab::PREFIX_DEITY));
    }

    #[test]
    fn convert_and_convert_text_are_identical_for_plain_words() {
        assert_eq!(convert("kalli"), convert_text("kalli"));
    }

    #[test]
    fn apply_f2m_pehki() {
        assert_eq!(apply_f2m("pehki"), "pewki");
    }

    #[test]
    fn apply_f2m_kohkeh() {
        assert_eq!(apply_f2m("kohkeh"), "kowkeh");
    }

    #[test]
    fn apply_f2m_nihneki() {
        assert_eq!(apply_f2m("nihneki"), "nikneki");
    }

    #[test]
    fn apply_f2m_tonatih() {
        assert_eq!(apply_f2m("tonatih"), "tonatiw");
    }

    #[test]
    fn apply_f2m_tzih() {
        assert_eq!(apply_f2m("τih"), "τin");
    }

    #[test]
    fn apply_f2m_michih() {
        assert_eq!(apply_f2m("miςih"), "miςin");
    }

    #[test]
    fn convert_all_basic() {
        let result = convert_all("kalli");

        assert!(!result.trager.is_empty());
        assert!(!result.atom.is_empty());
        assert!(!result.ipa.is_empty());
        assert!(!result.search.is_empty());
    }
}
