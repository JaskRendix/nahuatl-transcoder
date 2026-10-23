use once_cell::sync::Lazy;
use regex::Regex;

use crate::en_endings::ends_in_en;

static HK2WK: Lazy<Regex> = Lazy::new(|| {
    Regex::new(
        r"(ala|kana|ka|se|ςipa|ςi|ςo|sia|si|ko|κahκa|κite|e|wapa|itonalpote|li|lo|ma|maxihxiλa|mela|me|mole|ne|ni|paλa|pe|pina|piτa|pohye|poke|po|ki|tila|λaxλa|λiκeti|τa|τi|xa|xiλa|xi|ya|yo|sa|soλa|so)hk(i|eh)$"
    )
    .unwrap()
});

pub fn hk2wk(word: &str) -> String {
    HK2WK.replace(word, "${1}wk${2}").into_owned()
}

static HCONS2KCONS: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"([nt]i)h([mnptkκτλςsxhlwyñβdgfrρbv])([aeio])").unwrap());

pub fn h_cons_to_k_cons(word: &str) -> String {
    HCONS2KCONS.replace(word, "${1}k${2}${3}").into_owned()
}

pub fn h2w(word: &str) -> String {
    match word {
        "peτikih" => "peτikiw".to_string(),
        "tonatih" => "tonatiw".to_string(),
        "koneh" => "konew".to_string(),
        _ => word.to_string(),
    }
}

pub fn h2n(word: &str) -> String {
    let mut w = word.to_string();

    //
    // τih -> τin
    //

    if !w.ends_with("τahτih")
        && !w.ends_with("weτih")
        && let Some(stem) = w.strip_suffix("τih")
    {
        w = format!("{stem}τin");
    }

    //
    // ςih -> ςin
    //

    if !w.ends_with("leςih")
        && !w.ends_with("peςih")
        && let Some(stem) = w.strip_suffix("ςih")
    {
        w = format!("{stem}ςin");
    }

    //
    // lih -> lin
    //

    if !w.ends_with("kikilih")
        && let Some(stem) = w.strip_suffix("lih")
    {
        w = format!("{stem}lin");
    }

    //
    // Explicit exceptions from NOCE
    //

    w = w.replace("iniwah", "inihwan");
    w = w.replace("iniwantih", "inihwantin");

    w = w.replace("inmowah", "inmohwan");
    w = w.replace("inmowantih", "inmohwantin");

    w = w.replace("towah", "tohwan");
    w = w.replace("towantih", "tohwantin");

    //
    // en_endings.js
    //

    if ends_in_en(&w) && w.ends_with('h') {
        w.pop();
        w.push('n');
    }

    w
}

pub fn apply_f2m(word: &str) -> String {
    let word = hk2wk(word);
    let word = h_cons_to_k_cons(&word);
    let word = h2w(&word);
    h2n(&word)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hk2wk_pehki() {
        assert_eq!(hk2wk("pehki"), "pewki");
    }

    #[test]
    fn hk2wk_kohkeh() {
        assert_eq!(hk2wk("kohkeh"), "kowkeh");
    }

    #[test]
    fn hcons2kcons() {
        assert_eq!(h_cons_to_k_cons("nihneki"), "nikneki");
    }

    #[test]
    fn h2w_tonatih() {
        assert_eq!(h2w("tonatih"), "tonatiw");
    }

    #[test]
    fn h2n_tzih() {
        assert_eq!(h2n("τih"), "τin");
    }

    #[test]
    fn h2n_michih() {
        assert_eq!(h2n("miςih"), "miςin");
    }

    #[test]
    fn h2n_totolih() {
        assert_eq!(h2n("totolih"), "totolin");
    }
}
