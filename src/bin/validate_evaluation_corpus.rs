use std::env;
use std::process::ExitCode;

use nahuatl_transcoder::evaluation::validate_corpus_dir;

fn run() -> Result<(), ExitCode> {
    let mut args = env::args();
    let program = args
        .next()
        .unwrap_or_else(|| "validate_evaluation_corpus".into());
    let Some(directory) = args.next() else {
        eprintln!("Usage: {program} <corpus-directory>");
        return Err(ExitCode::from(2));
    };
    if args.next().is_some() {
        eprintln!("Usage: {program} <corpus-directory>");
        return Err(ExitCode::from(2));
    }

    match validate_corpus_dir(&directory) {
        Ok(corpus) => {
            println!(
                "Valid corpus {:?} version {}: {} record(s)",
                corpus.manifest.corpus_id,
                corpus.manifest.version,
                corpus.records.len()
            );
            Ok(())
        }
        Err(errors) => {
            for error in &errors {
                eprintln!("{error}");
            }
            eprintln!("Corpus validation failed with {} error(s).", errors.len());
            Err(ExitCode::FAILURE)
        }
    }
}

fn main() -> ExitCode {
    run().err().unwrap_or(ExitCode::SUCCESS)
}
