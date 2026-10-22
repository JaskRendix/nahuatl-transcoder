pub mod constants;
pub mod structure;

use constants::*;
use structure::structure;

pub fn convert(input: &str) -> String {
    let mut s = input.to_lowercase();

    s = asciify_vowels(s);
    s = convert_digraphs(s);
    s = convert_single_consonants(s);
    s = convert_vowels(s);

    structure(&s)
}

pub fn asciify_vowels(mut s: String) -> String {
    let replacements = [
        ("ā", "a"),
        ("á", "a"),
        ("à", "a"),
        ("ä", "a"),
        ("â", "a"),
        ("ã", "a"),
        ("ē", "e"),
        ("é", "e"),
        ("è", "e"),
        ("ë", "e"),
        ("ê", "e"),
        ("ī", "i"),
        ("í", "i"),
        ("ì", "i"),
        ("ï", "i"),
        ("î", "i"),
        ("ō", "o"),
        ("ó", "o"),
        ("ò", "o"),
        ("ö", "o"),
        ("ô", "o"),
        ("õ", "o"),
        ("ū", "u"),
        ("ú", "u"),
        ("ù", "u"),
        ("ü", "u"),
        ("û", "u"),
    ];

    for (from, to) in replacements {
        s = s.replace(from, to);
    }

    s
}

pub fn convert_digraphs(mut s: String) -> String {
    s = s.replace("cuh", &CONSONANT_CUA.to_string());
    s = s.replace("hu", &CONSONANT_WA.to_string());
    s = s.replace("uh", &CONSONANT_WA.to_string());
    s = s.replace("qu", &CONSONANT_CA.to_string());
    s = s.replace("cu", &CONSONANT_CUA.to_string());

    s = s.replace("ce", &format!("{}{}", CONSONANT_SA, VOWEL_E));
    s = s.replace("ci", &format!("{}{}", CONSONANT_SA, VOWEL_I));

    s = s.replace("ku", &CONSONANT_CUA.to_string());
    s = s.replace("kw", &CONSONANT_CUA.to_string());
    s = s.replace("uc", &CONSONANT_CUA.to_string());

    s = s.replace("tz", &CONSONANT_TZA.to_string());
    s = s.replace("ts", &CONSONANT_TZA.to_string());
    s = s.replace("tl", &CONSONANT_TLA.to_string());
    s = s.replace("ch", &CONSONANT_CHA.to_string());
    s = s.replace("rr", &CONSONANT_RRA.to_string());

    s = s.replace("ge", &format!("{}e", CONSONANT_HA));
    s = s.replace("gi", &format!("{}i", CONSONANT_HA));
    s = s.replace("jua", &format!("{}a", CONSONANT_WA));

    s
}

pub fn convert_single_consonants(mut s: String) -> String {
    let rules = [
        ("m", CONSONANT_MA),
        ("n", CONSONANT_NA),
        ("p", CONSONANT_PA),
        ("t", CONSONANT_TA),
        ("c", CONSONANT_CA),
        ("k", CONSONANT_CA),
        ("s", CONSONANT_SA),
        ("z", CONSONANT_SA),
        ("x", CONSONANT_XA),
        ("h", CONSONANT_HA),
        ("j", CONSONANT_HA),
        ("l", CONSONANT_LA),
        ("y", CONSONANT_YA),
        ("w", CONSONANT_WA),
        ("ñ", CONSONANT_NYA),
        ("v", CONSONANT_BVA),
        ("b", CONSONANT_BVA),
        ("d", CONSONANT_DA),
        ("g", CONSONANT_GA),
        ("f", CONSONANT_FA),
        ("r", CONSONANT_RA),
    ];

    for (from, to) in rules {
        s = s.replace(from, &to.to_string());
    }

    s
}

pub fn convert_vowels(mut s: String) -> String {
    s = s.replace("a", &VOWEL_A.to_string());
    s = s.replace("e", &VOWEL_E.to_string());
    s = s.replace("i", &VOWEL_I.to_string());
    s = s.replace("o", &VOWEL_O.to_string());
    s = s.replace("u", &VOWEL_U.to_string());

    s
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn atl() {
        assert_eq!(convert("atl"), "");
    }

    #[test]
    fn nahuatl() {
        assert_eq!(convert("nahuatl"), "");
    }

    #[test]
    fn calli() {
        assert_eq!(convert("calli"), "");
    }

    #[test]
    fn ocelotl() {
        assert_eq!(convert("ocelotl"), "");
    }

    #[test]
    fn cuextecatl() {
        assert_eq!(convert("cuextecatl"), "");
    }

    #[test]
    fn empty_string() {
        assert_eq!(convert(""), "");
    }

    #[test]
    fn accented_input() {
        assert!(!convert("ā").is_empty());
    }

    #[test]
    fn spanish_rr() {
        let out = convert("perro");
        assert!(!out.is_empty());
    }
}
