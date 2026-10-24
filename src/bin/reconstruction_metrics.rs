use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::{BufRead, BufReader};

use nahuatl_transcoder::{
    convert_all,
    reconstruction::{CandidateStatus, reconstruct_sep},
};

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

    // Atomic form -> representative headword.
    let atomic_to_headword: HashMap<String, String> = headwords
        .iter()
        .map(|word| {
            let conversion = convert_all(word);
            (conversion.atom, word.clone())
        })
        .collect();

    println!("Loaded {} entries", headwords.len());
    println!("Unique atomic forms {}", lexicon.len());

    let mut processed = 0usize;

    let mut total_candidates = 0usize;
    let mut total_lexicon_hits = 0usize;

    let mut original_present = 0usize;
    let mut original_unique_lexicon_hit = 0usize;
    let mut exact_unique_reconstruction = 0usize;

    let mut unique_reconstructions = 0usize;
    let mut ambiguous_reconstructions = 0usize;

    let mut max_candidates = 0usize;

    for word in &headwords {
        let conversion = convert_all(word);
        let reconstructed = reconstruct_sep(&conversion.sep);

        processed += 1;

        let candidate_count = reconstructed.candidates.len();

        total_candidates += candidate_count;
        max_candidates = max_candidates.max(candidate_count);

        match reconstructed.status {
            CandidateStatus::Unique => unique_reconstructions += 1,
            CandidateStatus::Ambiguous => ambiguous_reconstructions += 1,
            CandidateStatus::NoCandidates => {}
        }

        let original_atomic = &conversion.atom;

        let original_found = reconstructed
            .candidates
            .iter()
            .any(|candidate| candidate.atomic == *original_atomic);

        if original_found {
            original_present += 1;
        }

        if original_found && candidate_count == 1 {
            exact_unique_reconstruction += 1;
        }

        let lexicon_hits: Vec<_> = reconstructed
            .candidates
            .iter()
            .filter(|candidate| lexicon.contains(&candidate.atomic))
            .collect();

        total_lexicon_hits += lexicon_hits.len();

        if lexicon_hits.len() == 1 && lexicon_hits[0].atomic == *original_atomic {
            original_unique_lexicon_hit += 1;
        } else {
            println!();
            println!("========================================");
            println!("MULTIPLE LEXICON ANALYSES");
            println!("HEADWORD: {}", word);
            println!("SEP:      {}", conversion.sep);
            println!("ATOMIC:   {}", original_atomic);

            println!("\nLEXICON HITS:");

            for hit in &lexicon_hits {
                let headword = atomic_to_headword
                    .get(&hit.atomic)
                    .map(String::as_str)
                    .unwrap_or("?");

                println!(
                    "  {:<25} headword={:<25} derivations={:?}",
                    hit.atomic, headword, hit.derivations
                );
            }
        }
    }

    println!();
    println!("=== RECONSTRUCTION COVERAGE ===");

    println!(
        "Original recovered: {}/{} ({:.2}%)",
        original_present,
        processed,
        100.0 * original_present as f64 / processed as f64
    );

    println!(
        "Exact and unique: {}/{} ({:.2}%)",
        exact_unique_reconstruction,
        processed,
        100.0 * exact_unique_reconstruction as f64 / processed as f64
    );

    println!();
    println!("=== RECONSTRUCTION SHAPE ===");

    println!(
        "Unique reconstructions: {} ({:.2}%)",
        unique_reconstructions,
        100.0 * unique_reconstructions as f64 / processed as f64
    );

    println!(
        "Ambiguous reconstructions: {} ({:.2}%)",
        ambiguous_reconstructions,
        100.0 * ambiguous_reconstructions as f64 / processed as f64
    );

    println!(
        "Average candidates: {:.3}",
        total_candidates as f64 / processed as f64
    );

    println!("Max candidates: {}", max_candidates);

    println!();
    println!("=== LEXICON MATCHES ===");

    println!(
        "Average atomic lexicon matches: {:.3}",
        total_lexicon_hits as f64 / processed as f64
    );

    println!(
        "Original was sole lexicon hit: {}/{} ({:.2}%)",
        original_unique_lexicon_hit,
        processed,
        100.0 * original_unique_lexicon_hit as f64 / processed as f64
    );

    Ok(())
}
