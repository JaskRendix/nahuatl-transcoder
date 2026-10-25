# Nahuatl Transcoder (Rust)

A Rust implementation of Edward H. Trager's Nahuatl transcoding system, extended with concepts and mappings associated with the later [NOCE (Nahuatl Orthography Conversion Engine)](https://github.com/piratical/noce) project. It converts Latin-based Nahuatl spellings to a project-specific Atomic representation and the NahuatlOne abugida encoding, and supports conversion between Trager abugida and Atomic representations.

The main CLI output is checked against the checked-in `golden.csv` regression fixture. The repository does not include the original transcoder used to produce that fixture.

## Features

* Classical and modern Nahuatl spelling normalization
* Latin orthography → Atomic conversion
* Atomic → NahuatlOne abugida conversion
* Trager abugida ↔ Atomic conversion
* Multiple output forms through the `convert_all()` library API: Trager, ACK, SEP, Hasler, Atomic, allophones, direct IPA, phonetic IPA, and search
* Ambiguity-preserving SEP → Atomic/ACK reconstruction through `reconstruct_sep()`
* Name, place, deity, and immutable-word metadata in `convert_text()`
* Classical Nahuatl → modern SEP orthography converter (`c2m`)
* Unicode-aware Rust implementation
* Command-line and lexicon-diagnostic binaries
* Golden-output, metadata, targeted reconstruction, and synthetic round-trip tests

## Build

```bash
cargo build --release
```

The tracked `regenerate.sh` is a legacy helper that invokes `./convert`, which is not included in this repository; it is not part of the Cargo build or a runnable way to regenerate `golden.csv` from this checkout.

Run the test suite with the same command used by CI:

```bash
cargo test --all-features
```

## Usage

Convert a text file, with each line processed independently:

```bash
cargo run -- words.txt
```

Or run the compiled binary:

```bash
./target/release/nahuatl_transcoder words.txt
```

The `nahuatl_transcoder` CLI reads one file and converts each line to the Trager-style NahuatlOne abugida. It does not offer output-format flags. The library API exposes additional output forms:

```rust
use nahuatl_transcoder::convert_all;

let result = convert_all("kalli");
println!("{}", result.search);
```

`convert_all()` returns a `ConversionResult` with these fields:

* `trager`: Trager-style NahuatlOne abugida
* `ack`, `sep`, `hasler`: Latin-based orthographic forms
* `atom`: normalized Atomic representation used by this implementation
* `allo`: result of applying the implemented M→F allophone rules to `atom`
* `ipa`: segment-by-segment IPA rendering of `atom`
* `iph`: IPA rendering of `allo` after consonant degemination, syllabification, and stress marking
* `search`: M→F form with consonant gemination removed

`sep` and `hasler` are rendered from `allo`; `ack`, `trager`, `atom`, and `ipa` are derived from `atom`.

### SEP reconstruction

Conversion and reconstruction serve different purposes. `convert_all()` converts an input into its normalized Atomic form and derived outputs, including a SEP rendering. `reconstruct_sep()` works in the opposite direction: it takes a SEP form and generates plausible underlying Atomic forms, with an ACK rendering for each. Because phonetic SEP can merge distinctions present in ACK, reconstruction can return multiple candidates rather than choosing one underlying form.

```rust
use nahuatl_transcoder::reconstruction::reconstruct_sep;

let candidates = reconstruct_sep("nosij");
for candidate in &candidates.candidates {
    println!("{} -> {}", candidate.atomic, candidate.ack);
}
```

For `nosij`, the candidate set includes `nosih` → `nocih` and `nosin` → `nocin`: the first retains the normalized surface-shaped form, while the second is a possible analysis with final `n`. A `Candidate` exposes `atomic`, `ack`, and `derivations`. A `CandidateSet` exposes `candidates` and a `status` of `NoCandidates`, `Unique`, or `Ambiguous`.

Candidates are hypotheses, not linguistic judgments. The implementation filters candidates by applying `apply_m2f()` and requiring an exact match to the normalized SEP input or a match after consonant degemination. Identical Atomic forms are deduplicated and their derivation labels are merged.

Implemented derivation labels are:

* `Unchanged`: keep the normalized input as a candidate.
* `FToM`: apply the implemented F→M helper to the normalized input.
* `FinalNRecovery`: propose final `n` for an input ending in `h`.
* `FinalWRecovery`: propose final `w` for an input ending in vowel + `h`.
* `CodaWRecovery`: propose `w` for `h` between a vowel and a consonant; with multiple matching positions, generate single-position recoveries and a candidate changing all such positions.
* `KLenitionRecovery`: propose `k` for `h` in the implemented vowel–`h`–consonant–vowel context.
* `KwRecovery`: propose `κ` in the coda `k`/`h` contexts accepted by the implementation.
* `LlHlRecovery`: propose `ll` for `h` in the `ihl[a|i]` context corresponding to the forward `ill[a|i]` → `ihl[a|i]` rule.
* `Gemination`, `FToMThenGemination`, and `GeminationThenFToM`: apply the implemented lexical gemination patterns alone or in combination with F→M conversion.

`reconstruct_sep()` sorts candidates by the public `Candidate::score()` heuristic. It adds 10 for `Unchanged`, subtracts 1 for each of `FinalNRecovery` and `FinalWRecovery`, and subtracts 2 for each of `KLenitionRecovery` and `CodaWRecovery`; other derivation labels do not affect the score. This ordering is a rule-based heuristic, not a calibrated probability or an evaluated linguistic ranking guarantee.

The `reconstruction_cases.rs` integration test covers selected examples, combinations, and negative controls. The `reconstruction_corpus.rs` test checks round trips over `words.txt` and `golden.csv`, including projection, deduplication, ACK renderings, and recall. In the checked-in fixtures, it reports **100% synthetic round-trip candidate recall**: each fixture's original Atomic form, as produced by `convert_all()`, appears among the candidates reconstructed from that same conversion's SEP output. The test currently processes 295 entries, reports an average of 1.556 candidates per entry, and a maximum of 10. Repeated inputs count more than once. These are implementation-consistency results, not measurements of accuracy on independently attested SEP forms paired with reviewed underlying analyses. See [the reconstruction evaluation](docs/reconstruction-evaluation.md) for the test protocol and its limitations.

`convert_text()` converts whitespace-separated text to Trager-style output and recognizes metadata tags at the beginning of a token (after any leading punctuation). The tag constants are available in `metaword`:

* `TAG_PROPER_NAME` (`U+200B`): mark a proper name
* `TAG_PLACE_NAME` (`U+200C`): mark a place name
* `TAG_DEITY_NAME` (`U+200D`): mark a deity name
* `TAG_IMMUTABLE` (`U+200E`): pass the word through unchanged

Name, place, and deity tags select the corresponding Trager prefix. If none is supplied, words can be classified using the built-in given-name, surname, and deity lists and a locative-suffix heuristic. If several name tags are present, the output prefix priority is deity, then place, then proper name. Immutable tokens pass through unchanged. Punctuation attached to a token is retained, but whitespace is normalized to single spaces; original spacing and line breaks are not preserved.

## c2m

The repository includes a second user-facing command-line utility, `c2m`, which converts Classical Nahuatl orthography to modern SEP orthography.

Examples:

```text
quetzal  -> ketsal
ce       -> se
ci       -> si
cu       -> ku
huitzil  -> uitsil
```

Run it with:

```bash
cargo run --bin c2m -- words.txt
```

Or run the compiled binary:

```bash
./target/release/c2m words.txt
```

The converter lowercases each input line, replaces the explicitly listed precomposed accented and macron vowels with their unmarked forms, and preserves `ch` while applying these spelling mappings: `hu`/`uh` → `u`, `qu` → `k`, `tz` → `ts`, `ce`/`ci` → `se`/`si`, `cu`/`uc` → `ku`, and otherwise `c` → `k`, `h` → `j`, and `z` → `s`. The vowel handling is not a general Unicode normalization routine.

## Reconstruction diagnostics

Three additional binaries inspect the bundled `src/data/g2.nfc.txt` lexicon. Run them from the repository root:

```bash
cargo run --bin reconstruction_coverage
cargo run --bin reconstruction_metrics
cargo run --bin reconstruction_ranking
```

These are diagnostics over the bundled lexicon, not an independently annotated evaluation set. They parse headwords from the file and apply this project's own conversion and reconstruction functions. In the current run, the ranking binary processed 6,033 headwords and reported 88.56% top-1 and 98.28% top-3 recovery of the input-derived Atomic form. These figures should not be interpreted as real-world reconstruction accuracy; additional reported figures and caveats are recorded in [the lexical ambiguity notes](docs/lexical_ambiguities.md).

### Evaluation corpus validation

The offline validator checks a corpus directory containing a `manifest.json` and a `records.jsonl` file. Its serde-based schema and validation rules are implemented in `src/evaluation/`; it does not fetch sources.

```text
my-corpus/
├── manifest.json
└── records.jsonl
```

Each nonblank JSONL line is one evaluation record with its observed and normalized SEP strings, adjudicated analyses, dialect, source reference, provenance, license reference, and optional notes. The manifest declares the corpus and schema versions, normalization version, source and license registries, ambiguity classes, and permitted splits. The complete field definitions, status/split assumptions, and synthetic examples are in [the evaluation architecture](docs/independent-reconstruction-evaluation.md) and [tests/evaluation/valid_corpus/](tests/evaluation/valid_corpus/).

Run validation from the repository root:

```bash
cargo run --offline --bin validate_evaluation_corpus -- my-corpus
```

The command exits successfully only when the manifest and all records validate. Errors include the record identifier (or manifest/line marker), field path, and a human-readable explanation. All validation is local and offline.

### Reconstruction evaluation

Both benchmark CLIs use a shared prepared-corpus pipeline: corpus validation happens once, each eligible record's accepted Atomic analyses are normalized once, and `reconstruct_sep()` runs once per record. Reconstruction evaluation and every ranking ablation then reuse those candidate sets. Any accepted analysis can satisfy ranking accuracy. `insufficient_evidence` records are excluded from accuracy and candidate-set denominators and listed separately. Other statuses with accepted analyses are included.

The report includes top-1, top-3, and configurable top-k accuracy; macro and micro candidate recall and precision; exact-set coverage; unsupported candidate rate; average, median, and maximum candidate-set size; no-candidate, system-ambiguity, unique-candidate, and gold-ambiguity rates; and the same metrics by declared ambiguity class. Undefined ratios are `null` in JSON and `n/a` in Markdown. Candidate absence from an accepted set means unsupported by this annotation, not necessarily linguistically false. Projection matching is included as a diagnostic and is not evidence of an accepted analysis.

Run from the repository root. Both report paths are explicit; top-k defaults to 3:

```bash
mkdir -p results
cargo run --offline --bin evaluate_reconstruction -- my-corpus \
  --json results/baseline.json \
  --markdown results/baseline.md \
  --top-k 5
```

The machine-readable and Markdown reports include corpus/schema/normalization versions, run time, repository version, Git commit and working-tree cleanliness when available, metric definitions, record and exclusion counts, ambiguity-class metrics, and each record's ordered candidates, scores, derivations, projections, and accepted-analysis matches. The evaluator is deterministic for a fixed corpus and code revision except for run timestamp and repository-state metadata. It is offline-only and does not modify reconstruction or ranking behavior.

### Lexical-prior ranking experiment

Compare baseline, lexical-prior, heuristic-only, lexical-only, and combined rankings against an already validated evaluation corpus:

```bash
mkdir -p results
cargo run --offline --bin evaluate_ranking -- my-corpus \
  --lexicon src/data/g2.nfc.txt \
  --ranker combined \
  --json results/ranking.json \
  --markdown results/ranking.md \
  --top-k 3
```

The selected ranker determines the report's `final_rank`; the report also retains baseline, lexical-prior, and three ablation metrics, plus each candidate's ranks under all five selections, feature values, baseline heuristic score, and lexical score. Both reports share corpus, crate, timestamp, and Git metadata; ranking reports additionally record the lexical-resource path and SHA-256 checksum. The resource is loaded and hashed from the same byte snapshot. The baseline reproduces the existing `Candidate::score()` ordering. The combined score is `100 × lexical_score + heuristic_score`; exact Atomic lexicon matches contribute 2 lexical points and normalized matches contribute 1. Lexical-only ties use Atomic lexical order; baseline and combined ties retain stable candidate order. The lexical resource is a ranking prior, not an independent gold source: report its provenance and overlap with any evaluation sources, and do not interpret results as independent if sources overlap.

## Architecture

The conversion pipeline normalizes input to Atomic, then renders orthographic or phonetic outputs. `convert_all()` produces SEP and Hasler from the M→F result; the other returned forms are rendered from Atomic. SEP reconstruction is a separate inverse-candidate process and does not change the forward conversion pipeline.

The main modules are:

* `lib.rs`: public conversion functions and the `ConversionResult` API.
* `atomic.rs` and `orthography.rs`: Latin spelling normalization and Atomic-to-ACK, SEP, Hasler, and IPA rendering.
* `abugida.rs`, `structure.rs`, and `constants.rs`: character mapping, NahuatlOne structural composition, and encoded symbols.
* `trager.rs`: Trager-abugida/Atomic mapping in both directions.
* `allophones.rs`: M→F rules and F→M helper rules used in conversion and reconstruction.
* `reconstruction.rs`: candidate generation, forward projection validation, deduplication, derivation tracking, and heuristic ordering.
* `ranking.rs`: baseline and lexical-prior rankers plus deterministic ranking ablations; this module reorders generated candidates without changing reconstruction.
* `phonetics.rs`: degemination, syllabification, stress marking, phonetic IPA, and search forms.
* `geminate.rs`: implemented lexical gemination patterns and final-`n`/`h` exceptions used by the phonological helpers.
* `metaword.rs`: punctuation handling, metadata tags, name/deity lists, and locative-suffix classification.
* `evaluation/`: serde corpus structures, offline validation, set-valued reconstruction metrics, benchmark execution, and JSON/Markdown reporting.
* `src/bin/`: the `c2m` converter, three lexicon reconstruction diagnostics, the corpus validator, and the benchmark runner.

## Project layout

```text
.
├── .github/
│   └── workflows/
│       └── ci.yml
├── .gitignore
├── Cargo.toml
├── LICENSE
├── README.md
├── docs/
│   ├── lexical_ambiguities.md
│   ├── reconstruction-evaluation.md
│   ├── trager-phonology-notes-es.md
│   └── trager-phonology-reference.md
├── golden.csv
├── regenerate.sh
├── words.txt
├── src/
│   ├── abugida.rs
│   ├── allophones.rs
│   ├── atomic.rs
│   ├── constants.rs
│   ├── evaluation/
│   │   ├── benchmark.rs
│   │   ├── errors.rs
│   │   ├── metrics.rs
│   │   ├── mod.rs
│   │   ├── report.rs
│   │   ├── schema.rs
│   │   └── validation.rs
│   ├── geminate.rs
│   ├── lib.rs
│   ├── main.rs
│   ├── metaword.rs
│   ├── orthography.rs
│   ├── phonetics.rs
│   ├── ranking.rs
│   ├── reconstruction.rs
│   ├── structure.rs
│   ├── trager.rs
│   ├── bin/
│   │   ├── c2m.rs
│   │   ├── reconstruction_coverage.rs
│   │   ├── reconstruction_metrics.rs
│   │   ├── reconstruction_ranking.rs
│   │   ├── evaluate_ranking.rs
│   │   ├── evaluate_reconstruction.rs
│   │   └── validate_evaluation_corpus.rs
│   ├── data/
│   │   ├── deities.txt
│   │   ├── g2.nfc.txt
│   │   ├── given_names.txt
│   │   └── surnames.txt
└── tests/
    ├── evaluation.rs
    ├── evaluation_benchmark.rs
    ├── golden.rs
    ├── metaword.rs
    ├── reconstruction_cases.rs
    ├── reconstruction_corpus.rs
    └── evaluation/
        ├── valid_corpus/
        ├── duplicate_ids/
        ├── bad_status/
        ├── bad_ambiguity/
        └── missing_analysis/
```

## Background

The repository documents the implementation as a Rust port and extension of Edward H. Trager's Nahuatl transcoder, with later functionality associated with the [NOCE](https://github.com/piratical/noce) project. The phonology notes in `docs/` discuss the Chicontepec-area variety described by Trager; they are not evidence that the implemented rules generalize to all Nahuatl varieties.

## License and attribution

This project is distributed under the GNU General Public License, version 2 or later (GPL-2.0-or-later), as specified in `Cargo.toml` and `LICENSE`. The upstream relationship and attribution are described in the project materials; the provenance of bundled data and the golden fixture is not established by this repository alone.
