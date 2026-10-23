use nahuatl_transcoder::metaword::{
    has_locative_suffix, is_deity, is_family_name, is_given_name, is_proper_name,
    segregate_punctuation, split_to_metawords,
};

#[test]
fn test_segregate_plain_word() {
    let (pre, word, post) = segregate_punctuation("tlakatl");

    assert_eq!(pre, "");
    assert_eq!(word, "tlakatl");
    assert_eq!(post, "");
}

#[test]
fn test_segregate_prefix_punctuation() {
    let (pre, word, post) = segregate_punctuation("\"Tlakatl");

    assert_eq!(pre, "\"");
    assert_eq!(word, "Tlakatl");
    assert_eq!(post, "");
}

#[test]
fn test_segregate_postfix_punctuation() {
    let (pre, word, post) = segregate_punctuation("Tlakatl.");

    assert_eq!(pre, "");
    assert_eq!(word, "Tlakatl");
    assert_eq!(post, ".");
}

#[test]
fn test_segregate_both_sides() {
    let (pre, word, post) = segregate_punctuation("\"Tlakatl,\"");

    assert_eq!(pre, "\"");
    assert_eq!(word, "Tlakatl");
    assert_eq!(post, ",\"");
}

#[test]
fn test_locative_suffix_ko() {
    assert!(has_locative_suffix("mexiko"));
}

#[test]
fn test_locative_suffix_tlan() {
    assert!(has_locative_suffix("akaλan"));
}

#[test]
fn test_locative_suffix_tepek() {
    assert!(has_locative_suffix("tepek"));
}

#[test]
fn test_not_locative() {
    assert!(!has_locative_suffix("tlakatl"));
}

#[test]
fn test_given_name() {
    assert!(is_given_name("Aaron"));
}

#[test]
fn test_surname() {
    assert!(is_family_name("Abarca"));
}

#[test]
fn test_proper_name_given() {
    assert!(is_proper_name("Aaron"));
}

#[test]
fn test_proper_name_surname() {
    assert!(is_proper_name("Abarca"));
}

#[test]
fn test_not_proper_name() {
    assert!(!is_proper_name("tlakatl"));
}

#[test]
fn test_deity_quetzalcoatl() {
    assert!(is_deity("keτalkoaλ"));
}

#[test]
fn test_deity_tlaloc() {
    assert!(is_deity("λalok"));
}

#[test]
fn test_not_deity() {
    assert!(!is_deity("tlakatl"));
}

#[test]
fn test_metaword_creation() {
    let words = split_to_metawords("Tlakatl");

    assert_eq!(words.len(), 1);

    let mw = &words[0];

    assert_eq!(mw.word, "Tlakatl");
    assert!(mw.flic);
    assert!(!mw.is_deity_name);
    assert!(!mw.is_place_name);
}

#[test]
fn test_metaword_proper_name() {
    let words = split_to_metawords("Aaron");

    let mw = &words[0];

    assert!(mw.is_proper_name);
    assert!(mw.flic);
}

#[test]
fn test_metaword_place_name() {
    let words = split_to_metawords("Mexico");

    let mw = &words[0];

    assert!(mw.is_place_name);
}

#[test]
fn test_metaword_deity_name() {
    let words = split_to_metawords("Keτalkoaλ");

    let mw = &words[0];

    assert!(mw.is_deity_name);
}

#[test]
fn test_multiple_words() {
    let words = split_to_metawords("Aaron λalok Tlakatl");

    assert_eq!(words.len(), 3);

    assert!(words[0].is_proper_name);
    assert!(words[1].is_deity_name);
    assert!(!words[2].is_proper_name);
}
