use std::env;
use std::process::ExitCode;

use nahuatl_transcoder::evaluation::{evaluate_corpus, validate_corpus_dir};

fn usage(program: &str) {
    eprintln!(
        "Usage: {program} <corpus-directory> --json <report.json> --markdown <report.md> [--top-k K]"
    );
}

fn run() -> Result<(), ExitCode> {
    let mut args = env::args();
    let program = args
        .next()
        .unwrap_or_else(|| "evaluate_reconstruction".into());
    let Some(corpus_directory) = args.next() else {
        usage(&program);
        return Err(ExitCode::from(2));
    };

    let mut json_path = None;
    let mut markdown_path = None;
    let mut top_k = 3usize;
    while let Some(argument) = args.next() {
        match argument.as_str() {
            "--json" => json_path = args.next(),
            "--markdown" => markdown_path = args.next(),
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
    let (Some(json_path), Some(markdown_path)) = (json_path, markdown_path) else {
        usage(&program);
        return Err(ExitCode::from(2));
    };

    let corpus = match validate_corpus_dir(&corpus_directory) {
        Ok(corpus) => corpus,
        Err(errors) => {
            for error in errors {
                eprintln!("{error}");
            }
            eprintln!("Evaluation stopped because corpus validation failed.");
            return Err(ExitCode::FAILURE);
        }
    };

    let report = evaluate_corpus(&corpus, top_k);
    let json = serde_json::to_string_pretty(&report).map_err(|error| {
        eprintln!("Could not serialize evaluation report: {error}");
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
        "Evaluated {} eligible record(s); excluded {}. Wrote {} and {}.",
        report.metrics.record_count, report.excluded_record_count, json_path, markdown_path
    );
    Ok(())
}

fn main() -> ExitCode {
    run().err().unwrap_or(ExitCode::SUCCESS)
}
