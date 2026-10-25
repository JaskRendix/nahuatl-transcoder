use std::sync::OnceLock;

fn en_endings() -> &'static [&'static str] {
    static ENDINGS: OnceLock<Vec<&'static str>> = OnceLock::new();
    ENDINGS.get_or_init(|| {
        include_str!("data/en_endings.txt")
            .lines()
            .map(str::trim)
            .filter(|s| !s.is_empty() && !s.starts_with('#'))
            .collect()
    })
}

pub fn ends_in_en(word: &str) -> bool {
    en_endings().iter().any(|suffix| word.ends_with(suffix))
}
