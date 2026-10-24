use crate::trager::trager_to_atomic;

pub const ATOMIC_VOWELS: &str = "aeioāēīōuū";

pub const ATOMIC_CONSONANTS: &str = "mnptkκτλςsxhlwyñβdgfrρbv";

pub const ATOMIC_KW: char = 'κ';
pub const ATOMIC_TS: char = 'τ';
pub const ATOMIC_TL: char = 'λ';
pub const ATOMIC_CH: char = 'ς';
pub const ATOMIC_RR: char = 'ρ';

pub fn is_atomic_vowel(ch: char) -> bool {
    ATOMIC_VOWELS.contains(ch)
}

pub fn is_atomic_consonant(ch: char) -> bool {
    ATOMIC_CONSONANTS.contains(ch)
}

pub fn is_atomic_letter(ch: char) -> bool {
    is_atomic_vowel(ch) || is_atomic_consonant(ch)
}

pub fn is_foreign_consonant(ch: char) -> bool {
    matches!(ch, 'ñ' | 'β' | 'd' | 'g' | 'f' | 'r' | 'ρ' | 'b' | 'v')
}

pub fn to_unmarked_vowel(ch: char) -> char {
    match ch {
        'ā' => 'a',
        'ē' => 'e',
        'ī' => 'i',
        'ō' => 'o',
        'ū' => 'u',
        _ => ch,
    }
}

const GENERAL_TO_ATOMIC: &[(&str, &str)] = &[
    // historical
    ("cuh", "κ"),
    // colonial v-for-w spellings
    ("ava", "awa"),
    ("ave", "awe"),
    ("avi", "awi"),
    ("avo", "awo"),
    ("eva", "ewa"),
    ("eve", "ewe"),
    ("evi", "ewi"),
    ("evo", "ewo"),
    ("iva", "iwa"),
    ("ive", "iwe"),
    ("ivi", "iwi"),
    ("ivo", "iwo"),
    ("ova", "owa"),
    ("ove", "owe"),
    ("ovi", "owi"),
    ("ovo", "owo"),
    // colonial vowel+y spellings
    ("ym", "im"),
    ("yn", "in"),
    ("yp", "ip"),
    ("yt", "it"),
    ("yc", "ic"),
    ("yq", "iq"),
    ("yh", "ih"),
    ("ys", "is"),
    ("yl", "il"),
    ("yx", "ix"),
    ("yw", "iw"),
    // assimilation
    ("np", "mp"),
    // digraphs
    ("ch", "ς"),
    ("tz", "τ"),
    ("ts", "τ"),
    ("tl", "λ"),
    ("hu", "w"),
    ("uh", "w"),
    ("qu", "k"),
    ("cu", "κ"),
    ("uc", "κ"),
    // c disambiguation
    ("ca", "ka"),
    ("co", "ko"),
    ("ce", "se"),
    ("ci", "si"),
    ("cā", "kā"),
    ("cō", "kō"),
    ("cē", "sē"),
    ("cī", "sī"),
    // modern
    ("ku", "κ"),
    ("kw", "κ"),
    ("sh", "x"),
    // foreign consonants
    ("rr", "ρ"),
    // alternative spellings
    ("ç", "s"),
    ("z", "s"),
    // SEP
    ("u", "w"),
    ("j", "h"),
    // MUST BE LAST
    ("c", "k"),
];

pub fn to_atomic(input: &str) -> String {
    if input.is_empty() {
        return String::new();
    }

    let mut out = input.to_lowercase();

    for (from, to) in GENERAL_TO_ATOMIC {
        out = out.replace(from, to);
    }

    trager_to_atomic(&out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hu_to_w() {
        assert_eq!(to_atomic("hua"), "wa");
    }

    #[test]
    fn uh_to_w() {
        assert_eq!(to_atomic("auh"), "aw");
    }

    #[test]
    fn tz_to_tau() {
        assert_eq!(to_atomic("tzintli"), "τinλi");
    }

    #[test]
    fn ts_to_tau() {
        assert_eq!(to_atomic("tsintli"), "τinλi");
    }

    #[test]
    fn tl_to_lambda() {
        assert_eq!(to_atomic("atl"), "aλ");
    }

    #[test]
    fn ch_to_sigma() {
        assert_eq!(to_atomic("chichi"), "ςiςi");
    }

    #[test]
    fn cu_to_kappa() {
        assert_eq!(to_atomic("cualli"), "κalli");
    }

    #[test]
    fn qu_to_k() {
        assert_eq!(to_atomic("quetzal"), "keτal");
    }

    #[test]
    fn ce_ci_rules() {
        assert_eq!(to_atomic("ce"), "se");
        assert_eq!(to_atomic("ci"), "si");
    }

    #[test]
    fn rr_rule() {
        assert_eq!(to_atomic("perro"), "peρo");
        assert_eq!(to_atomic("guerra"), "gweρa");
    }

    #[test]
    fn colonial_v_for_w() {
        assert_eq!(to_atomic("ava"), "awa");
        assert_eq!(to_atomic("ove"), "owe");
    }

    #[test]
    fn sep_u_to_w() {
        assert_eq!(to_atomic("uatl"), "waλ");
    }

    #[test]
    fn sep_j_to_h() {
        assert_eq!(to_atomic("juan"), "hwan");
    }

    #[test]
    fn z_to_s() {
        assert_eq!(to_atomic("zan"), "san");
    }

    #[test]
    fn np_assimilation() {
        assert_eq!(to_atomic("panpa"), "pampa");
    }

    #[test]
    fn kw_to_kappa() {
        assert_eq!(to_atomic("kwalli"), "κalli");
    }

    #[test]
    fn chui_is_parsed_as_ch_plus_ui() {
        assert_eq!(to_atomic("pachuiliā"), "paςwiliā");
    }

    #[test]
    fn trager_is_processed() {
        let s = format!("{}", crate::constants::CONSONANT_MA);
        assert_eq!(to_atomic(&s), "ma");
    }
}
