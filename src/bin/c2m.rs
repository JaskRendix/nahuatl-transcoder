use std::env;
use std::fs::File;
use std::io::{BufRead, BufReader};

const VOWEL_REPLACEMENTS: &[(&str, &str)] = &[
    ("ā", "a"),
    ("á", "a"),
    ("à", "a"),
    ("ä", "a"),
    ("â", "a"),
    ("ã", "a"),
    ("é", "e"),
    ("ē", "e"),
    ("è", "e"),
    ("ë", "e"),
    ("ê", "e"),
    ("í", "i"),
    ("ī", "i"),
    ("ì", "i"),
    ("ï", "i"),
    ("î", "i"),
    ("ō", "o"),
    ("ó", "o"),
    ("ò", "o"),
    ("ö", "o"),
    ("ô", "o"),
    ("õ", "o"),
    ("ū", "u"),
    ("ú", "u"),
    ("ù", "u"),
    ("ü", "u"),
    ("û", "u"),
];

const DIGRAPHS: &[(&str, &str)] = &[
    ("hu", "u"),
    ("uh", "u"),
    ("qu", "k"),
    ("tz", "ts"),
    ("ce", "se"),
    ("ci", "si"),
    ("cu", "ku"),
    ("uc", "ku"),
];

const LETTER_REPLACEMENTS: &[(&str, &str)] = &[("c", "k"), ("h", "j"), ("z", "s")];

fn asciify_vowels(target: &mut String) {
    for &(from, to) in VOWEL_REPLACEMENTS {
        *target = target.replace(from, to);
    }
}

fn convert_line(mut target: String) -> String {
    target = target.to_lowercase();
    asciify_vowels(&mut target);

    // Protect "ch" from the separate "c" and "h" conversions.
    target = target.replace("ch", "ç");

    for &(from, to) in DIGRAPHS {
        target = target.replace(from, to);
    }

    for &(from, to) in LETTER_REPLACEMENTS {
        target = target.replace(from, to);
    }

    target.replace("ç", "ch")
}

fn run() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = env::args();
    let program = args.next().unwrap_or_else(|| "c2m".to_string());

    let filename = match (args.next(), args.next()) {
        (Some(filename), None) => filename,
        _ => {
            eprintln!("c2m: Convert Nahuatl text from classical to modern orthography.");
            eprintln!("     (c) 2016 by Edward H. Trager. All Rights Reserved");
            eprintln!();
            eprintln!(
                "c2m: Herramienta para convertir texto en náhuatl desde la ortografía clásica"
            );
            eprintln!("     a la ortografía moderna.");
            eprintln!();
            eprintln!(
                "Derechos de autor (c) 2016 por Edward H. Trager. Todos los derechos reservados."
            );
            eprintln!();
            eprintln!("GPL v. 2.0 or later. Licencia Pública General de GNU v. 2.0 o posterior");
            eprintln!();
            eprintln!("Usage: {program} <file to convert>");
            std::process::exit(1);
        }
    };

    let file = File::open(filename)?;
    let reader = BufReader::new(file);

    for line in reader.lines() {
        println!("{}", convert_line(line?));
    }

    Ok(())
}

fn main() {
    if let Err(err) = run() {
        eprintln!("c2m: {err}");
        std::process::exit(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn qu_to_k() {
        assert_eq!(convert_line("quetzal".into()), "ketsal");
    }

    #[test]
    fn hu_to_u() {
        assert_eq!(convert_line("huitzil".into()), "uitsil");
    }

    #[test]
    fn ce_to_se() {
        assert_eq!(convert_line("ce".into()), "se");
    }

    #[test]
    fn ci_to_si() {
        assert_eq!(convert_line("ci".into()), "si");
    }

    #[test]
    fn ch_preserved() {
        assert_eq!(convert_line("chichi".into()), "chichi");
    }

    #[test]
    fn cu_to_ku() {
        assert_eq!(convert_line("cu".into()), "ku");
    }

    #[test]
    fn uc_to_ku() {
        assert_eq!(convert_line("uc".into()), "ku");
    }

    #[test]
    fn accented_vowels() {
        assert_eq!(convert_line("ĀÉÍÓÚ".into()), "aeiou");
    }

    #[test]
    fn classical_word() {
        assert_eq!(convert_line("Cuauhtli".into()), "kuautli");
    }
}
