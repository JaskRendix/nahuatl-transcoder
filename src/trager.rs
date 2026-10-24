use crate::atomic::{is_atomic_consonant, is_atomic_letter, is_atomic_vowel};
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

        // 4. Long intrinsic-A vowels on consonant bases
        //
        // CONSONANT_MA + LONG_VOWEL_SIGN -> mā
        // CONSONANT_NA + LONG_VOWEL_SIGN -> nā
        // etc.
        //
        for &(c, base) in &consonants {
            rules.push((
                format!("{}{}", c, nab::LONG_VOWEL_SIGN),
                Box::leak(format!("{}ā", base).into_boxed_str()),
            ));
        }

        // 4b. Standalone vowels with long mark
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

fn trager_base(ch: char) -> char {
    match ch {
        // vowels
        'a' => nab::VOWEL_A,
        'e' => nab::VOWEL_E,
        'i' => nab::VOWEL_I,
        'o' => nab::VOWEL_O,
        'u' => nab::VOWEL_U,

        // long vowels
        // NOTE: these are handled elsewhere in atomic_to_trager(),
        // so reaching these is usually a bug.
        'ā' => nab::VOWEL_A,
        'ē' => nab::VOWEL_E,
        'ī' => nab::VOWEL_I,
        'ō' => nab::VOWEL_O,
        'ū' => nab::VOWEL_U,

        // native consonants
        'm' => nab::CONSONANT_MA,
        'n' => nab::CONSONANT_NA,
        'p' => nab::CONSONANT_PA,
        't' => nab::CONSONANT_TA,
        'k' => nab::CONSONANT_CA,
        'κ' => nab::CONSONANT_CUA,
        'τ' => nab::CONSONANT_TZA,
        'λ' => nab::CONSONANT_TLA,
        'ς' => nab::CONSONANT_CHA,
        's' => nab::CONSONANT_SA,
        'x' => nab::CONSONANT_XA,
        'h' => nab::CONSONANT_HA,
        'l' => nab::CONSONANT_LA,
        'y' => nab::CONSONANT_YA,
        'w' => nab::CONSONANT_WA,

        // foreign consonants
        'ñ' => nab::CONSONANT_NYA,
        'β' | 'b' | 'v' => nab::CONSONANT_BVA,
        'd' => nab::CONSONANT_DA,
        'g' => nab::CONSONANT_GA,
        'f' => nab::CONSONANT_FA,
        'r' => nab::CONSONANT_RA,
        'ρ' => nab::CONSONANT_RRA,

        _ => panic!("no Trager base mapping for {:?}", ch),
    }
}

fn unmarked_vowel(ch: char) -> char {
    match ch {
        'ā' => 'a',
        'ē' => 'e',
        'ī' => 'i',
        'ō' => 'o',
        'ū' => 'u',
        _ => ch,
    }
}

fn compound_vowel_sign(pair: &str) -> Option<char> {
    match pair {
        "ia" => Some(nab::VOWEL_SIGN_IA),
        "ai" => Some(nab::VOWEL_SIGN_AI),
        "oa" => Some(nab::VOWEL_SIGN_OA),
        "eo" => Some(nab::VOWEL_SIGN_EO),
        "ei" => Some(nab::VOWEL_SIGN_EI),
        "io" => Some(nab::VOWEL_SIGN_IO),
        "ao" => Some(nab::VOWEL_SIGN_AO),
        _ => None,
    }
}

pub fn atomic_to_trager(input: &str) -> String {
    let chars: Vec<char> = input.chars().collect();
    let mut out = String::new();

    for i in 0..chars.len() {
        let current = chars[i];

        let previous = if i > 0 { chars[i - 1] } else { ' ' };

        let next = if i + 1 < chars.len() {
            chars[i + 1]
        } else {
            ' '
        };

        //
        // atomic vowel after consonant, or word-initial h + vowel
        //
        if is_atomic_vowel(current)
            && ((is_atomic_consonant(previous) && previous != 'h')
                || (previous == 'h' && (i == 1 || !is_atomic_letter(chars[i - 2]))))
        {
            match current {
                'a' => {}

                'ā' => out.push(nab::LONG_VOWEL_SIGN),

                'e' => out.push(nab::VOWEL_SIGN_E),
                'i' => out.push(nab::VOWEL_SIGN_I),
                'o' => out.push(nab::VOWEL_SIGN_O),
                'u' => out.push(nab::VOWEL_SIGN_U),

                'ē' => {
                    out.push(nab::VOWEL_SIGN_E);
                    out.push(nab::LONG_VOWEL_SIGN);
                }

                'ī' => {
                    out.push(nab::VOWEL_SIGN_I);
                    out.push(nab::LONG_VOWEL_SIGN);
                }

                'ō' => {
                    out.push(nab::VOWEL_SIGN_O);
                    out.push(nab::LONG_VOWEL_SIGN);
                }

                'ū' => {
                    out.push(nab::VOWEL_SIGN_U);
                    out.push(nab::LONG_VOWEL_SIGN);
                }

                _ => {}
            }
        }
        //
        // terminal h
        //
        else if current == 'h' && is_atomic_vowel(previous) {
            out.push(nab::SUBJOINER_SIGN);
            out.push(trager_base('h'));
        }
        //
        // terminal consonant
        //
        else if is_atomic_consonant(current)
            && is_atomic_vowel(previous)
            && !is_atomic_vowel(next)
        {
            out.push(nab::SUBJOINER_SIGN);
            out.push(trager_base(current));
        }
        //
        // compound vowels
        //
        else if is_atomic_vowel(current)
            && is_atomic_vowel(previous)
            && i != 1
            && is_atomic_letter(chars[i - 2])
        {
            let pair = format!("{}{}", unmarked_vowel(previous), unmarked_vowel(current));

            if let Some(sign) = compound_vowel_sign(&pair) {
                if previous == 'a' {
                    out.push(sign);
                } else if previous == 'ā' {
                    out.pop(); // remove LONG_VOWEL_SIGN
                    out.push(sign);
                } else {
                    if unmarked_vowel(previous) == previous {
                        // previous was short vowel:
                        // remove the previous vowel sign
                        out.pop();
                    } else {
                        // previous was long vowel:
                        // remove vowel sign + LONG_VOWEL_SIGN
                        out.pop();
                        out.pop();
                    }

                    out.push(sign);
                }
            } else {
                match current {
                    'a' => {}
                    'ā' => {
                        out.push(nab::VOWEL_A);
                        out.push(nab::LONG_VOWEL_SIGN);
                    }
                    'e' => out.push(nab::VOWEL_E),
                    'i' => out.push(nab::VOWEL_I),
                    'o' => out.push(nab::VOWEL_O),
                    'u' => out.push(nab::VOWEL_U),
                    'ē' => {
                        out.push(nab::VOWEL_E);
                        out.push(nab::LONG_VOWEL_SIGN);
                    }
                    'ī' => {
                        out.push(nab::VOWEL_I);
                        out.push(nab::LONG_VOWEL_SIGN);
                    }
                    'ō' => {
                        out.push(nab::VOWEL_O);
                        out.push(nab::LONG_VOWEL_SIGN);
                    }
                    'ū' => {
                        out.push(nab::VOWEL_U);
                        out.push(nab::LONG_VOWEL_SIGN);
                    }
                    _ => {}
                }
            }
        }
        //
        // ordinary atomic letter
        //
        else if matches!(current, 'ā' | 'ē' | 'ī' | 'ō' | 'ū') {
            match current {
                'ā' => {
                    out.push(nab::VOWEL_A);
                    out.push(nab::LONG_VOWEL_SIGN);
                }
                'ē' => {
                    out.push(nab::VOWEL_E);
                    out.push(nab::LONG_VOWEL_SIGN);
                }
                'ī' => {
                    out.push(nab::VOWEL_I);
                    out.push(nab::LONG_VOWEL_SIGN);
                }
                'ō' => {
                    out.push(nab::VOWEL_O);
                    out.push(nab::LONG_VOWEL_SIGN);
                }
                'ū' => {
                    out.push(nab::VOWEL_U);
                    out.push(nab::LONG_VOWEL_SIGN);
                }
                _ => unreachable!(),
            }
        } else if is_atomic_letter(current) {
            out.push(trager_base(current));
        }
        //
        // punctuation
        //
        else {
            out.push(current);
        }
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

    #[test]
    fn roundtrip_simple_word() {
        let atomic = "κaw";

        let trager = atomic_to_trager(atomic);

        assert_eq!(trager_to_atomic(&trager), atomic);
    }

    #[test]
    fn roundtrip_long_vowels() {
        let atomic = "κāwλi";

        let trager = atomic_to_trager(atomic);

        assert_eq!(trager_to_atomic(&trager), atomic);
    }

    #[test]
    fn roundtrip_complex_word() {
        let atomic = "mikλanteκλi";

        let trager = atomic_to_trager(atomic);

        assert_eq!(trager_to_atomic(&trager), atomic);
    }

    #[test]
    fn every_encodable_rule_roundtrips() {
        for (trager, _) in get_trager_to_atomic_rules() {
            // subjoined consonants are decoder-only
            if trager.starts_with(nab::SUBJOINER_SIGN) {
                continue;
            }

            let atomic = trager_to_atomic(trager);
            let regenerated = atomic_to_trager(&atomic);

            assert_eq!(trager_to_atomic(&regenerated), atomic);
        }
    }

    #[test]
    fn roundtrip_kia() {
        let atomic = "kia";

        assert_eq!(trager_to_atomic(&atomic_to_trager(atomic)), atomic);
    }

    #[test]
    fn roundtrip_kai() {
        let atomic = "kai";

        assert_eq!(trager_to_atomic(&atomic_to_trager(atomic)), atomic);
    }

    #[test]
    fn roundtrip_koa() {
        let atomic = "koa";

        assert_eq!(trager_to_atomic(&atomic_to_trager(atomic)), atomic);
    }

    #[test]
    fn roundtrip_keo() {
        let atomic = "keo";

        assert_eq!(trager_to_atomic(&atomic_to_trager(atomic)), atomic);
    }

    #[test]
    fn roundtrip_kei() {
        let atomic = "kei";

        assert_eq!(trager_to_atomic(&atomic_to_trager(atomic)), atomic);
    }

    #[test]
    fn roundtrip_kio() {
        let atomic = "kio";

        assert_eq!(trager_to_atomic(&atomic_to_trager(atomic)), atomic);
    }

    #[test]
    fn roundtrip_kao() {
        let atomic = "kao";

        assert_eq!(trager_to_atomic(&atomic_to_trager(atomic)), atomic);
    }
}
