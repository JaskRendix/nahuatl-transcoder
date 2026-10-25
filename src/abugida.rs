//! Conversion from ATOMIC orthography to the Nahuatl abugida.
//!
//! Each ATOMIC letter is mapped directly to its corresponding
//! abugida symbol. Long vowels are represented as a vowel
//! character followed by the dedicated length marker.

use crate::constants::*;

/// Converts ATOMIC text into the Nahuatl abugida.
///
/// Unknown characters are passed through unchanged.
pub fn atomic_to_abugida(input: &str) -> String {
    let mut out = String::with_capacity(input.len());

    for ch in input.chars() {
        match ch {
            'a' => out.push(VOWEL_A),
            'e' => out.push(VOWEL_E),
            'i' => out.push(VOWEL_I),
            'o' => out.push(VOWEL_O),
            'u' => out.push(VOWEL_U),

            'ā' | 'ē' | 'ī' | 'ō' | 'ū' => {
                let base = match ch {
                    'ā' => VOWEL_A,
                    'ē' => VOWEL_E,
                    'ī' => VOWEL_I,
                    'ō' => VOWEL_O,
                    'ū' => VOWEL_U,
                    _ => unreachable!(),
                };
                out.push(base);
                out.push(LONG_VOWEL_SIGN);
            }

            'm' => out.push(CONSONANT_MA),
            'n' => out.push(CONSONANT_NA),
            'p' => out.push(CONSONANT_PA),
            't' => out.push(CONSONANT_TA),

            'k' => out.push(CONSONANT_CA),
            'κ' => out.push(CONSONANT_CUA),

            'τ' => out.push(CONSONANT_TZA),
            'λ' => out.push(CONSONANT_TLA),
            'ς' => out.push(CONSONANT_CHA),

            's' => out.push(CONSONANT_SA),
            'x' => out.push(CONSONANT_XA),

            'h' | 'ʔ' => out.push(CONSONANT_HA),

            'l' => out.push(CONSONANT_LA),
            'y' => out.push(CONSONANT_YA),
            'w' => out.push(CONSONANT_WA),

            'ñ' => out.push(CONSONANT_NYA),

            'β' | 'b' | 'v' => out.push(CONSONANT_BVA),

            'd' => out.push(CONSONANT_DA),
            'g' => out.push(CONSONANT_GA),
            'f' => out.push(CONSONANT_FA),

            'r' => out.push(CONSONANT_RA),
            'ρ' => out.push(CONSONANT_RRA),

            _ => out.push(ch),
        }
    }

    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn vowel_a() {
        assert_eq!(atomic_to_abugida("a"), VOWEL_A.to_string());
    }

    #[test]
    fn consonant_m() {
        assert_eq!(atomic_to_abugida("m"), CONSONANT_MA.to_string());
    }

    #[test]
    fn kw_maps() {
        assert_eq!(atomic_to_abugida("κ"), CONSONANT_CUA.to_string());
    }

    #[test]
    fn tl_maps() {
        assert_eq!(atomic_to_abugida("λ"), CONSONANT_TLA.to_string());
    }

    #[test]
    fn rr_maps() {
        assert_eq!(atomic_to_abugida("ρ"), CONSONANT_RRA.to_string());
    }

    #[test]
    fn long_a_maps() {
        let expected = format!("{}{}", VOWEL_A, LONG_VOWEL_SIGN);

        assert_eq!(atomic_to_abugida("ā"), expected);
    }

    #[test]
    fn word_maps() {
        let expected = format!(
            "{}{}{}{}{}",
            CONSONANT_CUA, VOWEL_A, CONSONANT_WA, CONSONANT_TLA, VOWEL_I
        );

        assert_eq!(atomic_to_abugida("κawλi"), expected);
    }

    #[test]
    fn long_e_maps() {
        let expected = format!("{}{}", VOWEL_E, LONG_VOWEL_SIGN);
        assert_eq!(atomic_to_abugida("ē"), expected);
    }

    #[test]
    fn long_i_maps() {
        let expected = format!("{}{}", VOWEL_I, LONG_VOWEL_SIGN);
        assert_eq!(atomic_to_abugida("ī"), expected);
    }

    #[test]
    fn long_o_maps() {
        let expected = format!("{}{}", VOWEL_O, LONG_VOWEL_SIGN);
        assert_eq!(atomic_to_abugida("ō"), expected);
    }

    #[test]
    fn long_u_maps() {
        let expected = format!("{}{}", VOWEL_U, LONG_VOWEL_SIGN);
        assert_eq!(atomic_to_abugida("ū"), expected);
    }

    #[test]
    fn nya_maps() {
        assert_eq!(atomic_to_abugida("ñ"), CONSONANT_NYA.to_string());
    }

    #[test]
    fn beta_maps() {
        assert_eq!(atomic_to_abugida("β"), CONSONANT_BVA.to_string());
    }
}
