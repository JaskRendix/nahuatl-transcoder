use std::fs::File;
use std::io::{BufRead, BufReader};

use nahuatl_transcoder::{convert_all, reconstruction::reconstruct_sep};

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

    let mut processed = 0usize;

    let mut top1 = 0usize;
    let mut top3 = 0usize;

    let mut total_rank = 0usize;
    let mut worst_rank = 0usize;

    let mut rank_1 = 0usize;
    let mut rank_2 = 0usize;
    let mut rank_3 = 0usize;
    let mut rank_4_plus = 0usize;

    let mut coverage_failures = Vec::new();

    for word in &headwords {
        let conversion = convert_all(word);

        let reconstructed = reconstruct_sep(&conversion.sep);

        processed += 1;

        let Some(rank) = reconstructed
            .candidates
            .iter()
            .position(|candidate| candidate.atomic == conversion.atom)
        else {
            coverage_failures.push(word.clone());
            continue;
        };

        let rank = rank + 1;

        total_rank += rank;
        worst_rank = worst_rank.max(rank);

        match rank {
            1 => rank_1 += 1,
            2 => rank_2 += 1,
            3 => rank_3 += 1,
            _ => rank_4_plus += 1,
        }

        if rank == 1 {
            top1 += 1;
        }

        if rank <= 3 {
            top3 += 1;
        }
    }

    println!();
    println!("=== RECONSTRUCTION RANKING ===");
    println!();

    println!("Processed entries: {}", processed);

    println!(
        "Top-1 accuracy: {}/{} ({:.2}%)",
        top1,
        processed,
        100.0 * top1 as f64 / processed as f64
    );

    println!(
        "Top-3 accuracy: {}/{} ({:.2}%)",
        top3,
        processed,
        100.0 * top3 as f64 / processed as f64
    );

    println!("Mean rank: {:.3}", total_rank as f64 / processed as f64);

    println!("Worst rank: {}", worst_rank);

    println!();

    println!("=== RANK DISTRIBUTION ===");
    println!(
        "Rank 1      : {} ({:.2}%)",
        rank_1,
        100.0 * rank_1 as f64 / processed as f64
    );

    println!(
        "Rank 2      : {} ({:.2}%)",
        rank_2,
        100.0 * rank_2 as f64 / processed as f64
    );

    println!(
        "Rank 3      : {} ({:.2}%)",
        rank_3,
        100.0 * rank_3 as f64 / processed as f64
    );

    println!(
        "Rank 4+     : {} ({:.2}%)",
        rank_4_plus,
        100.0 * rank_4_plus as f64 / processed as f64
    );

    if !coverage_failures.is_empty() {
        println!();
        println!("=== COVERAGE FAILURES ===");

        for word in coverage_failures {
            println!("{}", word);
        }
    }

    Ok(())
}
