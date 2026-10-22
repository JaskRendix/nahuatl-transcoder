use std::env;
use std::fs;

use nahuatl_transcoder::convert;

fn main() {
    let filename = env::args().nth(1).expect("usage: convert <file>");

    let content = fs::read_to_string(filename).expect("cannot read file");

    for line in content.lines() {
        println!("{}", convert(line));
    }
}
