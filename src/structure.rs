use crate::constants::*;

fn is_consonant(ch: char) -> bool {
    matches!(
        ch,
        CONSONANT_MA
            | CONSONANT_NA
            | CONSONANT_PA
            | CONSONANT_TA
            | CONSONANT_CA
            | CONSONANT_CUA
            | CONSONANT_TZA
            | CONSONANT_TLA
            | CONSONANT_CHA
            | CONSONANT_SA
            | CONSONANT_XA
            | CONSONANT_HA
            | CONSONANT_LA
            | CONSONANT_YA
            | CONSONANT_WA
            | CONSONANT_NYA
            | CONSONANT_BVA
            | CONSONANT_DA
            | CONSONANT_GA
            | CONSONANT_FA
            | CONSONANT_RA
            | CONSONANT_RRA
    )
}

fn is_vowel(ch: char) -> bool {
    matches!(ch, VOWEL_A | VOWEL_E | VOWEL_I | VOWEL_O | VOWEL_U)
}

fn is_vowel_u(ch: char) -> bool {
    ch == VOWEL_U
}

fn is_vowel_sign(ch: char) -> bool {
    matches!(
        ch,
        VOWEL_SIGN_A | VOWEL_SIGN_E | VOWEL_SIGN_I | VOWEL_SIGN_O | VOWEL_SIGN_U
    )
}

fn is_long_vowel_indicator(ch: char) -> bool {
    ch == LONG_VOWEL_SIGN
}

fn is_not_abugida(ch: char) -> bool {
    !(is_consonant(ch) || is_vowel(ch) || is_long_vowel_indicator(ch))
}

fn vowel_to_vowel_sign(ch: char) -> char {
    match ch {
        VOWEL_A => VOWEL_SIGN_A,
        VOWEL_E => VOWEL_SIGN_E,
        VOWEL_I => VOWEL_SIGN_I,
        VOWEL_O => VOWEL_SIGN_O,
        VOWEL_U => VOWEL_SIGN_U,
        _ => ch,
    }
}

pub fn structure(input: &str) -> String {
    let mut elem: Vec<char> = input.chars().collect();

    if elem.is_empty() {
        return input.to_string();
    }

    //
    // Phase Zero:
    // V U V -> V WA V
    //
    for i in 0..elem.len() {
        let first = elem.get(i).copied().unwrap_or('\0');
        let second = elem.get(i + 1).copied().unwrap_or('\0');
        let third = elem.get(i + 2).copied().unwrap_or('\0');

        if is_vowel(first) && is_vowel_u(second) && is_vowel(third) {
            elem[i + 1] = CONSONANT_WA;
        }
    }

    //
    // First phase
    //
    let mut i = 0;

    while i < elem.len() {
        let first = elem.get(i).copied().unwrap_or('\0');
        let second = elem.get(i + 1).copied().unwrap_or('\0');
        let third = elem.get(i + 2).copied().unwrap_or('\0');
        let fourth = elem.get(i + 3).copied().unwrap_or('\0');

        if is_consonant(first) && is_vowel(second) {
            elem[i + 1] = vowel_to_vowel_sign(second);

            if is_consonant(third) && (is_not_abugida(fourth) || is_consonant(fourth)) {
                elem.insert(i + 2, SUBJOINER_SIGN);
                i += 1;
            }
        }

        let first = elem.get(i).copied().unwrap_or('\0');
        let second = elem.get(i + 1).copied().unwrap_or('\0');
        let third = elem.get(i + 2).copied().unwrap_or('\0');

        if (is_vowel(first) || is_long_vowel_indicator(first))
            && is_consonant(second)
            && (is_not_abugida(third) || is_consonant(third))
        {
            elem.insert(i + 1, SUBJOINER_SIGN);
            i += 1;
        }

        i += 1;
    }

    //
    // Convert vowel sign + vowel (with optional long vowel markers)
    // into compound vowel signs
    //
    let mut i = 0;

    while i < elem.len() {
        let first = elem[i];
        if is_vowel_sign(first) {
            let mut vowel_pos = i + 1;
            // skip long marker after first vowel sign
            if vowel_pos < elem.len() && elem[vowel_pos] == LONG_VOWEL_SIGN {
                vowel_pos += 1;
            }
            if vowel_pos < elem.len() && is_vowel(elem[vowel_pos]) {
                let vowel = elem[vowel_pos];
                let replacement = match (first, vowel) {
                    (VOWEL_SIGN_I, VOWEL_A) => Some(VOWEL_SIGN_IA),
                    (VOWEL_SIGN_A, VOWEL_I) => Some(VOWEL_SIGN_AI),
                    (VOWEL_SIGN_O, VOWEL_A) => Some(VOWEL_SIGN_OA),
                    (VOWEL_SIGN_E, VOWEL_O) => Some(VOWEL_SIGN_EO),
                    (VOWEL_SIGN_E, VOWEL_I) => Some(VOWEL_SIGN_EI),
                    (VOWEL_SIGN_I, VOWEL_O) => Some(VOWEL_SIGN_IO),
                    (VOWEL_SIGN_A, VOWEL_O) => Some(VOWEL_SIGN_AO),
                    _ => None,
                };
                if let Some(compound) = replacement {
                    let mut end = vowel_pos + 1;
                    // skip long marker after second vowel
                    if end < elem.len() && elem[end] == LONG_VOWEL_SIGN {
                        end += 1;
                    }
                    elem.splice(i..end, [compound]);
                    continue;
                }
            }
        }
        i += 1;
    }

    //
    // Remove redundant vowel sign A
    //
    elem.retain(|c| *c != VOWEL_SIGN_A);

    elem.into_iter().collect()
}
