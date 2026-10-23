use std::collections::HashSet;

use nahuatl_transcoder::{
    apply_m2f, atomic::to_atomic, convert_all, orthography::atomic_to_ack, phonetics::degeminate,
    reconstruction::reconstruct_sep,
};

fn fixture_inputs() -> Vec<String> {
    let mut inputs: Vec<String> = include_str!("../words.txt")
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(str::to_owned)
        .collect();

    for (line_number, line) in include_str!("../golden.csv").lines().enumerate().skip(1) {
        let (input, output) = line
            .split_once("\",\"")
            .unwrap_or_else(|| panic!("malformed golden.csv row {}", line_number + 1));
        assert!(
            input.starts_with('"') && output.ends_with('"'),
            "malformed golden.csv row {}",
            line_number + 1
        );
        inputs.push(input.trim_start_matches('"').to_owned());
    }

    inputs
}

#[test]
fn existing_fixtures_report_reconstruction_metrics() {
    let inputs = fixture_inputs();
    let mut total_candidates = 0usize;
    let mut maximum_candidates = 0usize;
    let mut unique_results = 0usize;
    let mut ambiguous_results = 0usize;
    let mut no_candidate_results = 0usize;
    let mut recalled = 0usize;

    for input in &inputs {
        let expected = convert_all(input);
        let surface_atomic = to_atomic(&expected.sep);
        let result = reconstruct_sep(&expected.sep);
        let mut seen_candidates = HashSet::new();

        total_candidates += result.candidates.len();
        maximum_candidates = maximum_candidates.max(result.candidates.len());

        match result.candidates.len() {
            0 => no_candidate_results += 1,
            1 => unique_results += 1,
            _ => ambiguous_results += 1,
        }

        if result
            .candidates
            .iter()
            .any(|candidate| candidate.atomic == expected.atom)
        {
            recalled += 1;
        }

        for candidate in &result.candidates {
            assert!(
                seen_candidates.insert(candidate.atomic.as_str()),
                "duplicate Atomic candidate {:?} for fixture input {:?}",
                candidate.atomic,
                input
            );

            assert_eq!(
                candidate.ack,
                atomic_to_ack(&candidate.atomic),
                "incorrect ACK rendering for fixture input {:?}, candidate {:?}",
                input,
                candidate.atomic
            );

            let projected = apply_m2f(&candidate.atomic);
            assert!(
                projected == surface_atomic
                    || degeminate(&projected) == degeminate(&surface_atomic),
                "candidate {:?} fails projection for fixture input {:?}: projected {:?}, surface {:?}",
                candidate.atomic,
                input,
                projected,
                surface_atomic
            );
        }
    }

    let example_count = inputs.len();
    assert!(example_count > 0, "fixture corpus must contain examples");

    let percentage = |count: usize| 100.0 * count as f64 / example_count as f64;

    println!("examples: {example_count}");
    println!(
        "average candidates per word: {:.3}",
        total_candidates as f64 / example_count as f64
    );
    println!("maximum candidates per word: {maximum_candidates}");
    println!("unique results: {:.2}%", percentage(unique_results));
    println!("ambiguous results: {:.2}%", percentage(ambiguous_results));
    println!(
        "no-candidate results: {:.2}%",
        percentage(no_candidate_results)
    );
    println!("candidate recall: {:.2}%", percentage(recalled));
}
