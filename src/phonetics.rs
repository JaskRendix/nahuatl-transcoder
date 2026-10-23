use once_cell::sync::Lazy;
use regex::Regex;

//
// /ll/ -> [hl]
//

static LL2HL: Lazy<Regex> = Lazy::new(|| Regex::new(r"([i])l(l[ai])").unwrap());

pub fn ll2hl(word: &str) -> String {
    LL2HL.replace(word, "${1}h${2}").into_owned()
}

//
// terminal /n/ -> [h]
//

static N2H_FINAL: Lazy<Regex> = Lazy::new(|| Regex::new(r"n$").unwrap());

pub fn n2h(word: &str) -> String {
    N2H_FINAL.replace(word, "h").into_owned()
}

//
// /w/ coda -> [h]
//

static W2H: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"([aeio])w([mnptkκτλςsxhlwyñβdgfrρbv ])").unwrap());

static W2H_FINAL: Lazy<Regex> = Lazy::new(|| Regex::new(r"([aeio])w$").unwrap());

pub fn w2h(word: &str) -> String {
    let word = W2H.replace_all(word, "${1}h${2}");
    let word = W2H_FINAL.replace(&word, "${1}h");
    word.into_owned()
}

//
// /kʷ/ coda -> [h] before k
// /kʷ/ coda -> [k] elsewhere
//

static KW2H: Lazy<Regex> = Lazy::new(|| Regex::new(r"([aeio])κ(k)").unwrap());

static KW2K: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"([aeio])κ([mnptκτλςsxhlwyñβdgfrρbv ])").unwrap());

static KW2K_FINAL: Lazy<Regex> = Lazy::new(|| Regex::new(r"([aeio])κ$").unwrap());

pub fn kw2k_or_h(word: &str) -> String {
    let word = KW2H.replace_all(word, "${1}h${2}");
    let word = KW2K.replace_all(&word, "${1}k${2}");
    let word = KW2K_FINAL.replace(&word, "${1}k");

    word.into_owned()
}

//
// /k/ coda before consonant -> [h]
//

static KCONS2HCONS: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"([aeio])k([mnptkκτλςsxhlwyñβdgfrρbv])([aeio])").unwrap());

pub fn k_cons_to_h_cons(word: &str) -> String {
    KCONS2HCONS.replace(word, "${1}h${2}${3}").into_owned()
}

//
// Full M -> F pipeline
//

pub fn apply_m2f(word: &str) -> String {
    let word = ll2hl(word);
    let word = n2h(&word);
    let word = w2h(&word);
    let word = kw2k_or_h(&word);

    k_cons_to_h_cons(&word)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ll2hl_illia() {
        assert_eq!(ll2hl("illia"), "ihlia");
    }

    #[test]
    fn n2h_tzin() {
        assert_eq!(n2h("τin"), "τih");
    }

    #[test]
    fn w2h_tonatiw() {
        assert_eq!(w2h("tonatiw"), "tonatih");
    }

    #[test]
    fn kw2k_final() {
        assert_eq!(kw2k_or_h("aκ"), "ak");
    }

    #[test]
    fn kw2h_before_k() {
        assert_eq!(kw2k_or_h("aκka"), "ahka");
    }

    #[test]
    fn kcons2hcons() {
        assert_eq!(k_cons_to_h_cons("nikneki"), "nihneki");
    }

    #[test]
    fn apply_m2f_nihneki() {
        assert_eq!(apply_m2f("nikneki"), "nihneki");
    }

    #[test]
    fn apply_m2f_tonatiw() {
        assert_eq!(apply_m2f("tonatiw"), "tonatih");
    }

    #[test]
    fn apply_m2f_tzin() {
        assert_eq!(apply_m2f("τin"), "τih");
    }
}
