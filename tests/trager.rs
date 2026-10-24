use nahuatl_transcoder::constants::nab;
use nahuatl_transcoder::trager::{atomic_to_trager, get_trager_to_atomic_rules, trager_to_atomic};

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
