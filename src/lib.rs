pub mod abugida;
pub mod atomic;
pub mod constants;
pub mod geminate;
pub mod metaword;
pub mod orthography;
pub mod structure;
pub mod trager;

use abugida::atomic_to_abugida;
use atomic::to_atomic;
use metaword::split_to_metawords;
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
}
