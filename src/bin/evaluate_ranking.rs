use std::env;
use std::process::ExitCode;

use nahuatl_transcoder::{
    evaluation::validate_corpus_dir,
    ranking::{LexicalInventory, RankerSelection, compare_corpus_rankers_with_selection},
};

fn usage(program: &str) {
    eprintln!(
        "Usage: {program} <corpus-directory> --lexicon <g2-lexicon> --json <report.json> --markdown <report.md> [--top-k K] [--ranker baseline|lexical-prior|heuristic-only|lexical-only|combined]"
    );
}

fn run() -> Result<(), ExitCode> {
    let mut args = env::args();
    let program = args.next().unwrap_or_else(|| "evaluate_ranking".into());
    let Some(corpus_directory) = args.next() else {
        usage(&program);
        return Err(ExitCode::from(2));
    };
    let mut lexicon_path = None;
    let mut json_path = None;
    let mut markdown_path = None;
    let mut top_k = 3usize;
    let mut selected_ranker = RankerSelection::Combined;

    while let Some(argument) = args.next() {
        match argument.as_str() {
            "--lexicon" => lexicon_path = args.next(),
            "--json" => json_path = args.next(),
            "--markdown" => markdown_path = args.next(),
            "--ranker" => {
                let Some(value) = args.next() else {
                    usage(&program);
                    return Err(ExitCode::from(2));
                };
                selected_ranker = match value.as_str() {
                    "baseline" => RankerSelection::Baseline,
                    "lexical-prior" => RankerSelection::LexicalPrior,
                    "heuristic-only" => RankerSelection::HeuristicOnly,
                    "lexical-only" => RankerSelection::LexicalOnly,
                    "combined" => RankerSelection::Combined,
                    _ => {
                        eprintln!("Unknown ranker: {value}");
                        usage(&program);
                        return Err(ExitCode::from(2));
                    }
                };
            }
            "--top-k" => {
                let Some(value) = args.next() else {
                    usage(&program);
                    return Err(ExitCode::from(2));
                };
                top_k = match value.parse::<usize>() {
                    Ok(value) if value > 0 => value,
                    _ => {
                        eprintln!("--top-k must be a positive integer.");
                        usage(&program);
                        return Err(ExitCode::from(2));
                    }
                };
            }
            _ => {
                eprintln!("Unknown argument: {argument}");
                usage(&program);
                return Err(ExitCode::from(2));
            }
        }
    }
    let (Some(lexicon_path), Some(json_path), Some(markdown_path)) =
        (lexicon_path, json_path, markdown_path)
    else {
        usage(&program);
        return Err(ExitCode::from(2));
    };

    let corpus = match validate_corpus_dir(corpus_directory) {
        Ok(corpus) => corpus,
        Err(errors) => {
            for error in errors {
                eprintln!("{error}");
            }
            eprintln!("Ranking comparison stopped because corpus validation failed.");
            return Err(ExitCode::FAILURE);
        }
    };
    let inventory = LexicalInventory::load_g2(&lexicon_path).map_err(|error| {
        eprintln!("Could not load lexical resource {}: {error}", lexicon_path);
        ExitCode::FAILURE
    })?;
    let report = compare_corpus_rankers_with_selection(&corpus, &inventory, top_k, selected_ranker);
    let json = serde_json::to_string_pretty(&report).map_err(|error| {
        eprintln!("Could not serialize ranking comparison: {error}");
        ExitCode::FAILURE
    })?;
    let markdown = report.to_markdown();
    std::fs::write(&json_path, json).map_err(|error| {
        eprintln!("Could not write JSON report {}: {error}", json_path);
        ExitCode::FAILURE
    })?;
    std::fs::write(&markdown_path, markdown).map_err(|error| {
        eprintln!("Could not write Markdown report {}: {error}", markdown_path);
        ExitCode::FAILURE
    })?;
    println!(
        "Compared rankers on {} eligible record(s) using {} lexicon form(s); wrote {} and {}.",
        report.total_record_count - report.excluded_record_count,
        report.lexical_entry_count,
        json_path,
        markdown_path
    );
    Ok(())
}

fn main() -> ExitCode {
    run().err().unwrap_or(ExitCode::SUCCESS)
}
