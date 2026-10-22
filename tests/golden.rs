use std::fs;

use nahuatl_transcoder::convert;

#[test]
fn golden_not_empty() {
    let csv = std::fs::read_to_string("golden.csv").unwrap();
    assert!(csv.lines().count() > 100);
}

#[test]
fn golden_has_expected_size() {
    let csv = std::fs::read_to_string("golden.csv").unwrap();
    assert!(csv.lines().count() > 100);
}

#[test]
fn golden_corpus() {
    let csv = fs::read_to_string("golden.csv").expect("cannot read golden.csv");

    for line in csv.lines().skip(1) {
        let parts: Vec<&str> = line.split("\",\"").collect();

        if parts.len() != 2 {
            continue;
        }

        let input = parts[0].trim_start_matches('"');
        let expected = parts[1].trim_end_matches('"');

        let actual = convert(input);

        assert_eq!(actual, expected, "failed on input: {input}");
    }
}
