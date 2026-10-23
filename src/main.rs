use std::env;
use std::fs::File;
use std::io::{BufRead, BufReader};

use nahuatl_transcoder::convert;

fn run() -> Result<(), Box<dyn std::error::Error>> {
    let filename = env::args()
        .nth(1)
        .ok_or("usage: nahuatl_transcoder <file>")?;

    let file = File::open(&filename)?;
    let reader = BufReader::new(file);

    for line in reader.lines() {
        println!("{}", convert(&line?));
    }

    Ok(())
}

fn main() {
    if let Err(err) = run() {
        eprintln!("nahuatl_transcoder: {err}");
        std::process::exit(1);
    }
}
