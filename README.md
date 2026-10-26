# Nahuatl Transcoder (Rust)

A Rust port of Edward H. Trager's Nahuatl transcoding system, extended with concepts from the [NOCE](https://github.com/piratical/noce) project. It converts Latin-based Nahuatl spellings into a normalized Atomic representation, the NahuatlOne abugida, and various orthographic or phonetic forms.

## Features

* Classical and modern Nahuatl spelling normalization
* Latin orthography → Atomic conversion
* Atomic → NahuatlOne abugida conversion
* Trager abugida ↔ Atomic conversion
* Multiple output formats via `convert_all()`: Trager, ACK, SEP, Hasler, Atomic, allophones, IPA, and search
* Ambiguity-preserving SEP → Atomic/ACK reconstruction via `reconstruct_sep()`
* Proper name, place, deity, and immutable-word metadata handling in `convert_text()`
* Classical Nahuatl → modern SEP converter (`c2m`)
* Unicode-aware Rust core with regression tests

## Build & Test

Build the library and CLIs:

```bash
cargo build --release
```

Run tests:

```bash
cargo test
```

## Usage

Process a text file line-by-line:

```bash
cargo run -- words.txt
```

### Library API

```rust
use nahuatl_transcoder::convert_all;

let result = convert_all("kalli");
println!("{}", result.search);
```

`convert_all()` returns a `ConversionResult` struct containing:

* `trager`: Trager-style NahuatlOne abugida
* `ack`, `sep`, `hasler`: Latin-based orthographic forms
* `atom`: Normalized Atomic representation
* `allo`: Allophonic form (M→F rules applied)
* `ipa` / `iph`: Segment and phonetic IPA renderings
* `search`: Search-optimized form

### SEP Reconstruction

Phonetic SEP forms can merge underlying distinctions (like final `-h` vs `-n`). `reconstruct_sep()` generates plausible underlying Atomic candidates instead of guessing a single form:

```rust
use nahuatl_transcoder::reconstruction::reconstruct_sep;

let candidates = reconstruct_sep("nosij");
for candidate in &candidates.candidates {
    println!("{} -> {}", candidate.atomic, candidate.ack);
}
```

### C2M Converter

Convert Classical Nahuatl spellings to modern SEP orthography:

```bash
cargo run --bin c2m -- words.txt
```

## Project Layout

```text
.
├── Cargo.toml
├── LICENSE
├── README.md
├── golden.csv
├── words.txt
├── src/
│   ├── abugida.rs
│   ├── allophones.rs
│   ├── atomic.rs
│   ├── constants.rs
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
│   │   └── main.rs
│   └── data/
│       ├── deities.txt
│       ├── en_endings.txt
│       ├── g2.nfc.txt
│       ├── given_names.txt
│       └── surnames.txt
└── tests/
    ├── golden.rs
    ├── metaword.rs
    ├── reconstruction_cases.rs
    ├── reconstruction_corpus.rs
    └── trager.rs
```

## License

Distributed under the GNU General Public License, version 2 or later (GPL-2.0-or-later). See `LICENSE` for details.
