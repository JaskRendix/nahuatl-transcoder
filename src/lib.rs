pub mod abugida;
pub mod allophones;
pub mod atomic;
pub mod constants;
pub mod en_endings;
pub mod geminate;
pub mod metaword;
pub mod orthography;
pub mod phonetics;
pub mod structure;
pub mod trager;

use abugida::atomic_to_abugida;
use allophones::*;
use atomic::to_atomic;
use metaword::split_to_metawords;
use phonetics::*;
use structure::structure;

pub fn convert(input: &str) -> String {
    let atomic = to_atomic(input);

    let raw_abugida = atomic_to_abugida(&atomic);

    structure(&raw_abugida)
}

pub fn convert_text(input: &str) -> String {
    let words = split_to_metawords(input);

    let mut out = String::new();

    for mw in words {
        let abugida = structure(&atomic_to_abugida(&mw.atomic));

        if mw.is_deity_name {
            out.push(constants::nab::PREFIX_DEITY);
        } else if mw.is_place_name {
            out.push(constants::nab::PREFIX_PLACE);
        } else if mw.is_proper_name {
            out.push(constants::nab::PREFIX_NAME);
        }

        out.push_str(&abugida);
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
}
