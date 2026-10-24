use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::{BufRead, BufReader};

use nahuatl_transcoder::{atomic::to_atomic, convert_all, reconstruction::reconstruct_sep};

fn load_lexicon() -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let file = File::open("src/data/g2.nfc.txt")?;
    let reader = BufReader::new(file);

    let mut words = Vec::new();

    for line in reader.lines() {
        let line = line?;
        let line = line.trim();

        if line.is_empty() {
            continue;
        }

        let Some(headword) = line.split('.').next() else {
            continue;
        };

        let headword = headword.trim();

        if headword.is_empty() || headword.contains(' ') {
            continue;
        }

        words.push(headword.to_string());
    }

    Ok(words)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let headwords = load_lexicon()?;

    // Atomic forms present in the lexicon.
    let lexicon: HashSet<String> = headwords
        .iter()
        .map(|word| convert_all(word).atom)
        .collect();

    // Atomic form -> headword lookup.
    let lexicon_lookup: HashMap<String, String> = headwords
        .iter()
        .map(|word| {
            let conversion = convert_all(word);
            (conversion.atom, word.clone())
        })
        .collect();

    let mut misses = 0usize;

    for word in &headwords {
        let conversion = convert_all(word);

        let candidate_set = reconstruct_sep(&conversion.sep);

        let original_atomic = &conversion.atom;

        let recovered = candidate_set
            .candidates
            .iter()
            .any(|c| c.atomic == *original_atomic);

        if recovered {
            continue;
        }

        misses += 1;

        println!();
        println!("==================================================");
        println!("HEADWORD: {}", word);
        println!("SEP:      {}", conversion.sep);
        println!("ATOMIC:   {}", conversion.atom);
        println!("ACK:      {}", conversion.ack);
        println!("STATUS:   {:?}", candidate_set.status);
        println!("\nRECONSTRUCTION TRACE:");
        println!("  observed atomic = {}", to_atomic(&conversion.sep));
        println!("  target atomic   = {}", original_atomic);
        println!(
            "  target -> m2f   = {}",
            nahuatl_transcoder::apply_m2f(original_atomic)
        );
        println!("\nCANDIDATES:");

        for candidate in &candidate_set.candidates {
            let in_lexicon = lexicon.contains(&candidate.atomic);

            if let Some(headword) = lexicon_lookup.get(&candidate.atomic) {
                println!(
                    "  {:<25} ack={:<25} lexicon={} headword={:<25} derivations={:?}",
                    candidate.atomic, candidate.ack, in_lexicon, headword, candidate.derivations
                );
            } else {
                println!(
                    "  {:<25} ack={:<25} lexicon={} derivations={:?}",
                    candidate.atomic, candidate.ack, in_lexicon, candidate.derivations
                );
            }
        }
    }

    println!();
    println!("========================================");
    println!("TOTAL MISSES: {}", misses);

    Ok(())
}
