// src/constants.rs

pub mod nab {
    //
    // Vowels
    //
    pub const VOWEL_A: char = '\u{ED90}';
    pub const VOWEL_E: char = '\u{ED91}';
    pub const VOWEL_I: char = '\u{ED92}';
    pub const VOWEL_O: char = '\u{ED93}';
    pub const VOWEL_U: char = '\u{ED94}';

    //
    // Long vowel sign
    //
    pub const LONG_VOWEL_SIGN: char = '\u{ED95}';

    //
    // Vowel signs
    //
    pub const VOWEL_SIGN_A: char = '\u{EDA0}';
    pub const VOWEL_SIGN_E: char = '\u{EDA1}';
    pub const VOWEL_SIGN_I: char = '\u{EDA2}';
    pub const VOWEL_SIGN_O: char = '\u{EDA3}';
    pub const VOWEL_SIGN_U: char = '\u{EDA4}';

    //
    // Compound vowel signs
    //
    pub const VOWEL_SIGN_IA: char = '\u{EDA5}';
    pub const VOWEL_SIGN_AI: char = '\u{EDA6}';
    pub const VOWEL_SIGN_OA: char = '\u{EDA7}';
    pub const VOWEL_SIGN_EO: char = '\u{EDA8}';
    pub const VOWEL_SIGN_EI: char = '\u{EDA9}';
    pub const VOWEL_SIGN_IO: char = '\u{EDAA}';
    pub const SUBJOINER_SIGN: char = '\u{EDAB}';
    pub const VOWEL_SIGN_AO: char = '\u{EDAC}';

    //
    // Special prefixes
    //
    pub const PREFIX_PLACE: char = '\u{EDAD}';
    pub const PREFIX_NAME: char = '\u{EDAE}';
    pub const PREFIX_DEITY: char = '\u{EDAF}';

    //
    // Native consonants
    //
    pub const CONSONANT_MA: char = '\u{EDB0}';
    pub const CONSONANT_NA: char = '\u{EDB1}';
    pub const CONSONANT_PA: char = '\u{EDB2}';
    pub const CONSONANT_TA: char = '\u{EDB3}';
    pub const CONSONANT_CA: char = '\u{EDB4}';
    pub const CONSONANT_CUA: char = '\u{EDB5}';
    pub const CONSONANT_TZA: char = '\u{EDB6}';
    pub const CONSONANT_TLA: char = '\u{EDB7}';
    pub const CONSONANT_CHA: char = '\u{EDB8}';
    pub const CONSONANT_SA: char = '\u{EDB9}';
    pub const CONSONANT_XA: char = '\u{EDBA}';
    pub const CONSONANT_HA: char = '\u{EDBB}';
    pub const CONSONANT_LA: char = '\u{EDBC}';
    pub const CONSONANT_YA: char = '\u{EDBD}';
    pub const CONSONANT_WA: char = '\u{EDBE}';

    //
    // Spanish / foreign consonants
    //
    pub const CONSONANT_NYA: char = '\u{EDC0}';
    pub const CONSONANT_BVA: char = '\u{EDC1}';
    pub const CONSONANT_DA: char = '\u{EDC2}';
    pub const CONSONANT_GA: char = '\u{EDC3}';
    pub const CONSONANT_FA: char = '\u{EDC4}';
    pub const CONSONANT_RA: char = '\u{EDC5}';
    pub const CONSONANT_RRA: char = '\u{EDC6}';
}

pub use nab::*;
