use crate::constants::nab;
use std::sync::OnceLock;

/// Returns the ordered list of trager-to-atomic mapping rules.
/// Longest keys (e.g. consonant + vowel sign combinations) are placed first.
pub fn get_trager_to_atomic_rules() -> &'static [(String, &'static str)] {
    static RULES: OnceLock<Vec<(String, &'static str)>> = OnceLock::new();
    RULES.get_or_init(|| {
        let consonants = [
            (nab::CONSONANT_MA, "m"),
            (nab::CONSONANT_NA, "n"),
            (nab::CONSONANT_PA, "p"),
            (nab::CONSONANT_TA, "t"),
            (nab::CONSONANT_CA, "k"),
            (nab::CONSONANT_CUA, "κ"),
            (nab::CONSONANT_TZA, "τ"),
            (nab::CONSONANT_TLA, "λ"),
            (nab::CONSONANT_CHA, "ς"),
            (nab::CONSONANT_SA, "s"),
            (nab::CONSONANT_LA, "l"),
            (nab::CONSONANT_XA, "x"),
            (nab::CONSONANT_HA, "h"),
            (nab::CONSONANT_YA, "y"),
            (nab::CONSONANT_WA, "w"),
            (nab::CONSONANT_NYA, "ñ"),
            (nab::CONSONANT_BVA, "β"),
            (nab::CONSONANT_DA, "d"),
            (nab::CONSONANT_GA, "g"),
            (nab::CONSONANT_FA, "f"),
            (nab::CONSONANT_RA, "r"),
            (nab::CONSONANT_RRA, "ρ"),
        ];

        let mut rules = Vec::new();

        // 1. Diphthong & complex vowel signs following consonants
        for &(c, base) in &consonants {
            rules.push((
                format!("{}{}", c, nab::VOWEL_SIGN_IA),
                Box::leak(format!("{}ia", base).into_boxed_str()) as &str,
            ));
        }
        for &(c, base) in &consonants {
            rules.push((
                format!("{}{}", c, nab::VOWEL_SIGN_AI),
                Box::leak(format!("{}ai", base).into_boxed_str()) as &str,
            ));
        }
        for &(c, base) in &consonants {
            rules.push((
                format!("{}{}", c, nab::VOWEL_SIGN_OA),
                Box::leak(format!("{}oa", base).into_boxed_str()) as &str,
            ));
        }
        for &(c, base) in &consonants {
            rules.push((
                format!("{}{}", c, nab::VOWEL_SIGN_EO),
                Box::leak(format!("{}eo", base).into_boxed_str()) as &str,
            ));
        }
        for &(c, base) in &consonants {
            rules.push((
                format!("{}{}", c, nab::VOWEL_SIGN_EI),
                Box::leak(format!("{}ei", base).into_boxed_str()) as &str,
            ));
        }
        for &(c, base) in &consonants {
            rules.push((
                format!("{}{}", c, nab::VOWEL_SIGN_IO),
                Box::leak(format!("{}io", base).into_boxed_str()) as &str,
            ));
        }
        for &(c, base) in &consonants {
            rules.push((
                format!("{}{}", c, nab::VOWEL_SIGN_AO),
                Box::leak(format!("{}ao", base).into_boxed_str()) as &str,
            ));
        }

        // 2. Standard vowel signs (E, I, O, U) following consonants
        let vowel_signs = [
            (nab::VOWEL_SIGN_E, "e"),
            (nab::VOWEL_SIGN_I, "i"),
            (nab::VOWEL_SIGN_O, "o"),
            (nab::VOWEL_SIGN_U, "u"),
        ];
        for &(vs, v_str) in &vowel_signs {
            for &(c, base) in &consonants {
                rules.push((
                    format!("{}{}", c, vs),
                    Box::leak(format!("{}{}", base, v_str).into_boxed_str()) as &str,
                ));
            }
        }

        // 3. Subjoined Native Consonants
        for &(c, base) in &consonants {
            rules.push((
                format!("{}{}", nab::SUBJOINER_SIGN, c),
                Box::leak(base.to_string().into_boxed_str()) as &str,
            ));
        }

        // 4. Long Vowel Mappings
        let vowels = [
            nab::VOWEL_A,
            nab::VOWEL_E,
            nab::VOWEL_I,
            nab::VOWEL_O,
            nab::VOWEL_U,
        ];
        let long_vowel_chars = ["ā", "ē", "ī", "ō", "ū"];
        for (&v, &lv) in vowels.iter().zip(long_vowel_chars.iter()) {
            rules.push((format!("{}{}", v, nab::LONG_VOWEL_SIGN), lv));
        }
        rules.push((
            format!("{}{}", nab::VOWEL_SIGN_A, nab::LONG_VOWEL_SIGN),
            "ā",
        ));
        for &(vs, v_str) in &vowel_signs {
            let lv = match v_str {
                "e" => "ē",
                "i" => "ī",
                "o" => "ō",
                "u" => "ū",
                _ => unreachable!(),
            };
            rules.push((format!("{}{}", vs, nab::LONG_VOWEL_SIGN), lv));
        }

        // 5. Short or Unmarked Vowels & Vowel Signs
        rules.push((nab::VOWEL_A.to_string(), "a"));
        rules.push((nab::VOWEL_E.to_string(), "e"));
        rules.push((nab::VOWEL_I.to_string(), "i"));
        rules.push((nab::VOWEL_O.to_string(), "o"));
        rules.push((nab::VOWEL_U.to_string(), "u"));

        rules.push((nab::VOWEL_SIGN_A.to_string(), "a"));
        rules.push((nab::VOWEL_SIGN_E.to_string(), "e"));
        rules.push((nab::VOWEL_SIGN_I.to_string(), "i"));
        rules.push((nab::VOWEL_SIGN_O.to_string(), "o"));
        rules.push((nab::VOWEL_SIGN_U.to_string(), "u"));

        // 6. Compound (Diphthong) Vowel Signs standalone
        rules.push((nab::VOWEL_SIGN_IA.to_string(), "ia"));
        rules.push((nab::VOWEL_SIGN_AI.to_string(), "ai"));
        rules.push((nab::VOWEL_SIGN_OA.to_string(), "oa"));
        rules.push((nab::VOWEL_SIGN_EO.to_string(), "eo"));
        rules.push((nab::VOWEL_SIGN_EI.to_string(), "ei"));
        rules.push((nab::VOWEL_SIGN_IO.to_string(), "io"));
        rules.push((nab::VOWEL_SIGN_AO.to_string(), "ao"));

        // 7. Native Consonants with Intrinsic Vowel A (Single character)
        for &(c, base) in &consonants {
            rules.push((
                c.to_string(),
                Box::leak(format!("{}a", base).into_boxed_str()) as &str,
            ));
        }

        // 8. Subjoiner and Special Prefixes (Stripped out)
        rules.push((nab::SUBJOINER_SIGN.to_string(), ""));
        rules.push((nab::PREFIX_PLACE.to_string(), ""));
        rules.push((nab::PREFIX_NAME.to_string(), ""));
        rules.push((nab::PREFIX_DEITY.to_string(), ""));

        rules
    })
}

pub fn trager_to_atomic(input: &str) -> String {
    if input.is_empty() {
        return String::new();
    }

    let mut out = input.to_string();
    let rules = get_trager_to_atomic_rules();

    for (from, to) in rules {
        out = out.replace(from, to);
    }

    out
}

#[cfg(test)]
mod tests {
    use super::nab;
    use super::*;

    #[test]
    fn empty_string() {
        assert_eq!(trager_to_atomic(""), "");
    }

    //
    // Standalone vowels
    //
    #[test]
    fn vowel_a() {
        assert_eq!(trager_to_atomic(&nab::VOWEL_A.to_string()), "a");
    }

    #[test]
    fn vowel_e() {
        assert_eq!(trager_to_atomic(&nab::VOWEL_E.to_string()), "e");
    }

    #[test]
    fn vowel_i() {
        assert_eq!(trager_to_atomic(&nab::VOWEL_I.to_string()), "i");
    }

    #[test]
    fn vowel_o() {
        assert_eq!(trager_to_atomic(&nab::VOWEL_O.to_string()), "o");
    }

    #[test]
    fn vowel_u() {
        assert_eq!(trager_to_atomic(&nab::VOWEL_U.to_string()), "u");
    }

    //
    // Long vowels
    //
    #[test]
    fn long_vowel_a() {
        let s = format!("{}{}", nab::VOWEL_A, nab::LONG_VOWEL_SIGN);
        assert_eq!(trager_to_atomic(&s), "ā");
    }

    #[test]
    fn long_vowel_e() {
        let s = format!("{}{}", nab::VOWEL_E, nab::LONG_VOWEL_SIGN);
        assert_eq!(trager_to_atomic(&s), "ē");
    }

    #[test]
    fn long_vowel_i() {
        let s = format!("{}{}", nab::VOWEL_I, nab::LONG_VOWEL_SIGN);
        assert_eq!(trager_to_atomic(&s), "ī");
    }

    #[test]
    fn long_vowel_o() {
        let s = format!("{}{}", nab::VOWEL_O, nab::LONG_VOWEL_SIGN);
        assert_eq!(trager_to_atomic(&s), "ō");
    }

    #[test]
    fn long_vowel_u() {
        let s = format!("{}{}", nab::VOWEL_U, nab::LONG_VOWEL_SIGN);
        assert_eq!(trager_to_atomic(&s), "ū");
    }

    //
    // Consonants with intrinsic vowel A
    //
    #[test]
    fn consonant_ma() {
        assert_eq!(trager_to_atomic(&nab::CONSONANT_MA.to_string()), "ma");
    }

    #[test]
    fn consonant_cua() {
        assert_eq!(trager_to_atomic(&nab::CONSONANT_CUA.to_string()), "κa");
    }

    #[test]
    fn consonant_tza() {
        assert_eq!(trager_to_atomic(&nab::CONSONANT_TZA.to_string()), "τa");
    }

    #[test]
    fn consonant_tla() {
        assert_eq!(trager_to_atomic(&nab::CONSONANT_TLA.to_string()), "λa");
    }

    #[test]
    fn consonant_cha() {
        assert_eq!(trager_to_atomic(&nab::CONSONANT_CHA.to_string()), "ςa");
    }

    #[test]
    fn consonant_rra() {
        assert_eq!(trager_to_atomic(&nab::CONSONANT_RRA.to_string()), "ρa");
    }

    //
    // Standard vowel signs
    //
    #[test]
    fn ma_plus_e() {
        let s = format!("{}{}", nab::CONSONANT_MA, nab::VOWEL_SIGN_E);
        assert_eq!(trager_to_atomic(&s), "me");
    }

    #[test]
    fn ma_plus_i() {
        let s = format!("{}{}", nab::CONSONANT_MA, nab::VOWEL_SIGN_I);
        assert_eq!(trager_to_atomic(&s), "mi");
    }

    #[test]
    fn ma_plus_o() {
        let s = format!("{}{}", nab::CONSONANT_MA, nab::VOWEL_SIGN_O);
        assert_eq!(trager_to_atomic(&s), "mo");
    }

    #[test]
    fn ma_plus_u() {
        let s = format!("{}{}", nab::CONSONANT_MA, nab::VOWEL_SIGN_U);
        assert_eq!(trager_to_atomic(&s), "mu");
    }

    #[test]
    fn kua_plus_e() {
        let s = format!("{}{}", nab::CONSONANT_CUA, nab::VOWEL_SIGN_E);
        assert_eq!(trager_to_atomic(&s), "κe");
    }

    //
    // Compound vowel signs
    //
    #[test]
    fn ma_plus_ia() {
        let s = format!("{}{}", nab::CONSONANT_MA, nab::VOWEL_SIGN_IA);
        assert_eq!(trager_to_atomic(&s), "mia");
    }

    #[test]
    fn ma_plus_ai() {
        let s = format!("{}{}", nab::CONSONANT_MA, nab::VOWEL_SIGN_AI);
        assert_eq!(trager_to_atomic(&s), "mai");
    }

    #[test]
    fn ma_plus_oa() {
        let s = format!("{}{}", nab::CONSONANT_MA, nab::VOWEL_SIGN_OA);
        assert_eq!(trager_to_atomic(&s), "moa");
    }

    #[test]
    fn ma_plus_eo() {
        let s = format!("{}{}", nab::CONSONANT_MA, nab::VOWEL_SIGN_EO);
        assert_eq!(trager_to_atomic(&s), "meo");
    }

    #[test]
    fn ma_plus_ei() {
        let s = format!("{}{}", nab::CONSONANT_MA, nab::VOWEL_SIGN_EI);
        assert_eq!(trager_to_atomic(&s), "mei");
    }

    #[test]
    fn ma_plus_io() {
        let s = format!("{}{}", nab::CONSONANT_MA, nab::VOWEL_SIGN_IO);
        assert_eq!(trager_to_atomic(&s), "mio");
    }

    #[test]
    fn ma_plus_ao() {
        let s = format!("{}{}", nab::CONSONANT_MA, nab::VOWEL_SIGN_AO);
        assert_eq!(trager_to_atomic(&s), "mao");
    }

    //
    // Standalone compound vowel signs
    //
    #[test]
    fn standalone_ia() {
        assert_eq!(trager_to_atomic(&nab::VOWEL_SIGN_IA.to_string()), "ia");
    }

    #[test]
    fn standalone_ai() {
        assert_eq!(trager_to_atomic(&nab::VOWEL_SIGN_AI.to_string()), "ai");
    }

    //
    // Subjoined consonants
    //
    #[test]
    fn subjoined_m() {
        let s = format!("{}{}", nab::SUBJOINER_SIGN, nab::CONSONANT_MA);
        assert_eq!(trager_to_atomic(&s), "m");
    }

    #[test]
    fn subjoined_kw() {
        let s = format!("{}{}", nab::SUBJOINER_SIGN, nab::CONSONANT_CUA);
        assert_eq!(trager_to_atomic(&s), "κ");
    }

    #[test]
    fn subjoined_tl() {
        let s = format!("{}{}", nab::SUBJOINER_SIGN, nab::CONSONANT_TLA);
        assert_eq!(trager_to_atomic(&s), "λ");
    }

    #[test]
    fn subjoined_rr() {
        let s = format!("{}{}", nab::SUBJOINER_SIGN, nab::CONSONANT_RRA);
        assert_eq!(trager_to_atomic(&s), "ρ");
    }

    //
    // Prefix stripping
    //
    #[test]
    fn strips_place_prefix() {
        let s = format!("{}{}", nab::PREFIX_PLACE, nab::CONSONANT_MA);
        assert_eq!(trager_to_atomic(&s), "ma");
    }

    #[test]
    fn strips_name_prefix() {
        let s = format!("{}{}", nab::PREFIX_NAME, nab::CONSONANT_MA);
        assert_eq!(trager_to_atomic(&s), "ma");
    }

    #[test]
    fn strips_deity_prefix() {
        let s = format!("{}{}", nab::PREFIX_DEITY, nab::CONSONANT_MA);
        assert_eq!(trager_to_atomic(&s), "ma");
    }

    //
    // Multi-symbol integration tests
    //
    #[test]
    fn word_kawi() {
        let s = format!(
            "{}{}{}",
            nab::CONSONANT_CUA,
            nab::SUBJOINER_SIGN,
            nab::CONSONANT_WA
        );

        assert_eq!(trager_to_atomic(&s), "κaw");
    }

    #[test]
    fn word_tlakatl_like() {
        let s = format!(
            "{}{}{}",
            nab::CONSONANT_TLA,
            nab::CONSONANT_CA,
            nab::SUBJOINER_SIGN
        );

        assert!(trager_to_atomic(&s).starts_with("λaka"));
    }

    //
    // Unknown characters pass through
    //
    #[test]
    fn unknown_character_passthrough() {
        assert_eq!(trager_to_atomic("ABC"), "ABC");
    }
}
