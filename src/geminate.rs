use regex::Regex;

struct GemRule {
    pattern: &'static str,
    replacement: &'static str,
    exclude: Option<&'static str>,
}

static RULES: &[GemRule] = &[
    // M
    GemRule {
        pattern: r"^kema[hn]?$",
        replacement: "kemman",
        exclude: None,
    },
    GemRule {
        pattern: r"kema",
        replacement: "kemma",
        exclude: None,
    },
    GemRule {
        pattern: r"semo",
        replacement: "semmo",
        exclude: None,
    },
    // N
    GemRule {
        pattern: r"teno",
        replacement: "tenno",
        exclude: None,
    },
    // T
    GemRule {
        pattern: r"ita",
        replacement: "itta",
        exclude: Some(r"itaρ|ikxita|palomita"),
    },
    // K
    GemRule {
        pattern: r"seko",
        replacement: "sekko",
        exclude: None,
    },
    // KW
    GemRule {
        pattern: r"neκa",
        replacement: "neκκa",
        exclude: None,
    },
    // TZ
    GemRule {
        pattern: r"kotoτi",
        replacement: "kotoττi",
        exclude: None,
    },
    GemRule {
        pattern: r"κeτi",
        replacement: "κeττi",
        exclude: None,
    },
    GemRule {
        pattern: r"neτo",
        replacement: "neττo",
        exclude: None,
    },
    GemRule {
        pattern: r"wiτo",
        replacement: "wiττo",
        exclude: None,
    },
    GemRule {
        pattern: r"temeτon",
        replacement: "temeττon",
        exclude: None,
    },
    // S
    GemRule {
        pattern: r"^eso",
        replacement: "esso",
        exclude: None,
    },
    // L
    GemRule {
        pattern: r"^milah?$",
        replacement: "millah",
        exclude: None,
    },
    GemRule {
        pattern: r"^mili$",
        replacement: "milli",
        exclude: None,
    },
    GemRule {
        pattern: r"ali$",
        replacement: "alli",
        exclude: Some(r"piyali$"),
    },
    GemRule {
        pattern: r"eli$",
        replacement: "elli",
        exclude: Some(r"weli$"),
    },
    GemRule {
        pattern: r"ili$",
        replacement: "illi",
        exclude: Some(r"^(κamahmanili|wēwēyakilwili|λaκalλālili)$"),
    },
    GemRule {
        pattern: r"oli$",
        replacement: "olli",
        exclude: None,
    },
    // W
    GemRule {
        pattern: r"κatewi",
        replacement: "κatewwi",
        exclude: None,
    },
    GemRule {
        pattern: r"āλawahkaλan",
        replacement: "āλawwahkaλan",
        exclude: None,
    },
];

pub fn geminate(input: &str) -> String {
    let mut result = input.to_string();

    for rule in RULES {
        if let Some(exclude) = rule.exclude {
            let exclude = Regex::new(exclude).unwrap();

            if exclude.is_match(&result) {
                continue;
            }
        }

        let pattern = Regex::new(rule.pattern).unwrap();

        result = pattern.replace_all(&result, rule.replacement).into_owned();
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn geminate_kema() {
        assert_eq!(geminate("kema"), "kemman");
    }

    #[test]
    fn geminate_kemah() {
        assert_eq!(geminate("kemah"), "kemman");
    }

    #[test]
    fn geminate_semo() {
        assert_eq!(geminate("semo"), "semmo");
    }

    #[test]
    fn geminate_teno() {
        assert_eq!(geminate("teno"), "tenno");
    }

    #[test]
    fn geminate_ita() {
        assert_eq!(geminate("ita"), "itta");
    }

    #[test]
    fn dont_geminate_palomita() {
        assert_eq!(geminate("palomita"), "palomita");
    }

    #[test]
    fn geminate_neka() {
        assert_eq!(geminate("neκa"), "neκκa");
    }

    #[test]
    fn geminate_mili() {
        assert_eq!(geminate("mili"), "milli");
    }

    #[test]
    fn geminate_ali() {
        assert_eq!(geminate("kuali"), "kualli");
    }

    #[test]
    fn dont_geminate_piyali() {
        assert_eq!(geminate("piyali"), "piyali");
    }

    #[test]
    fn geminate_oli() {
        assert_eq!(geminate("yoli"), "yolli");
    }

    #[test]
    fn geminate_katewi() {
        assert_eq!(geminate("κatewi"), "κatewwi");
    }

    #[test]
    fn geminate_eso() {
        assert_eq!(geminate("eso"), "esso");
    }

    #[test]
    fn dont_geminate_weli() {
        assert_eq!(geminate("weli"), "weli");
    }

    #[test]
    fn geminate_milah() {
        assert_eq!(geminate("milah"), "millah");
    }
}
