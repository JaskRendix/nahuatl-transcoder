# SEP Reconstruction Evaluation

## What it does

SEP reconstruction addresses information lost when an Atomic form is written in a shallower, phonetic orthography. A single SEP form can match more than one underlying form, so `reconstruct_sep()` returns a `CandidateSet` rather than choosing one answer.

Each candidate includes:
- An Atomic form
- Its ACK rendering
- The derivation paths that produced it

This differs from conversion: `convert_all()` converts an input into outputs, including SEP. `reconstruct_sep()` takes SEP as input and proposes compatible underlying forms.

Candidates pass a projection check through the existing `apply_m2f()` implementation. This confirms compatibility with the code’s forward rules; it does not prove that a candidate is linguistically correct.

## Current synthetic benchmark

The `reconstruction_corpus.rs` integration test uses entries from `words.txt` and `golden.csv`. For each entry, it:

1. Calls `convert_all(input)` and gets its Atomic form and SEP output.
2. Passes the SEP form to `reconstruct_sep()`.
3. Counts a recall hit if the original Atomic form appears in the candidate set.
4. Checks candidate uniqueness, ACK rendering, and forward projection. Projection may use the existing degemination equivalence.

Run it with:

```bash
cargo test --test reconstruction_corpus -- --nocapture
```

The benchmark has 295 fixture entries and 148 distinct input forms. Repeated inputs count more than once, so the entry-level percentages are not a representative sample.

| Reconstruction stage | Synthetic candidate recall |
|---|---:|
| Initial MVP | 87.80% |
| After `FinalNRecovery` | 91.19% |
| After `CodaWRecovery` | 95.25% |
| After `LlHlRecovery` | 97.29% |
| After `KLenitionRecovery` | 98.64% |
| After `KwRecovery` | 100.00% |

The stage-wise percentages are retained from earlier documented benchmark runs. The repository does not include the intermediate implementations needed to reproduce each stage, so these values are not presented as measurements of the current implementation.

Current metrics:
- Average candidate-set size: **1.556**
- Maximum candidate-set size: **10**
- No-candidate rate: **0.00%**
- Candidate recall: **100.00%**

Recall does not show whether every candidate is plausible or whether the candidate sets are precise.

## Recovery rules

- **`FinalNRecovery`** — proposes word-final `n` when the surface form ends in `h`, based on the documented final `n` → `[h]` process.
- **`CodaWRecovery`** — proposes coda `w` where the forward process can surface it as `h`.
- **`LlHlRecovery`** — proposes `ll` in the context where the forward rule changes `ill[a|i]` to `ihl[a|i]`.
- **`KLenitionRecovery`** — proposes `k` from surface `h` in the context of the forward coda-`k` lenition rule.
- **`KwRecovery`** — proposes underlying `κ` where forward conversion can produce coda `k` or `h`, including the relevant context before `k`.

The listed recovery transforms generate candidates; candidate ordering happens separately. Each proposed candidate must pass the forward projection check.

After candidate generation and deduplication, `reconstruct_sep()` orders candidates by `Candidate::score()`. The score adds 10 for `Unchanged`, subtracts 1 for each `FinalNRecovery` and `FinalWRecovery` label, and subtracts 2 for each `KLenitionRecovery` and `CodaWRecovery` label. The other derivation labels do not affect the score. This is a heuristic ordering, not a calibrated probability or an independently validated measure of linguistic likelihood.

## Test coverage

The two reconstruction integration tests serve different purposes:

- **`reconstruction_cases.rs`** tests selected phonological examples, negative controls, rule combinations, and cases where ambiguity should remain. It documents rule behavior; it does not estimate corpus-wide accuracy.
- **`reconstruction_corpus.rs`** tests round trips across the current fixtures. It reports candidate counts and synthetic recall, and checks projection, deduplication, and ACK rendering.

Neither test uses an independently annotated collection of naturally occurring SEP forms paired with underlying analyses.

## What 100% synthetic recall means

The reported figure measures this round trip:

```text
original form → convert_all() → generated SEP → reconstruct_sep()
```

For every fixture entry, the Atomic form produced by `convert_all()` appears among the candidates generated from that same conversion’s SEP output.

It does **not** mean that 100% of real SEP forms can be reconstructed correctly. The surface forms and expected underlying forms both come from the project’s conversion path. The benchmark tests whether inverse candidate generation covers the forward rules used by these fixtures. Since projection validation uses the same forward implementation, it checks internal consistency, not independent linguistic accuracy.

The current test run reports an average of 1.556 candidates per fixture entry and a maximum of 10 (295 entries total). These values describe the candidate-set sizes produced for this fixture, not candidate precision or expected ambiguity in naturally occurring SEP data.

Other limits:
- `words.txt` and `golden.csv` are conversion and regression fixtures, not a representative SEP corpus with underlying-form annotations.
- Duplicate inputs count more than once.
- The fixtures may not cover dialect, spelling, morphological, or lexical variation in real materials.
- Recall does not penalize unsupported candidates. Candidate counts measure set size, not precision.
- A successful projection does not establish historical, morphological, or dialectal correctness.
- Each example needs provenance and linguistic support to establish that its rule applies.

## What a future evaluation corpus needs

A real evaluation corpus should pair SEP forms from independent sources with human-reviewed ACK or Atomic analyses. Each record should include:

- The SEP spelling as it appears in the source, plus any normalized form used for evaluation
- One or more acceptable underlying analyses, including ambiguity where the source does not resolve it
- Lexeme or lemma identity and available morphological context, such as affixes or compound boundaries
- Dialect or variety, when known
- Source citation and location, transcription and normalization notes, and license or permission details
- A process label, such as final `n` → `h` or coda `w` → `h`, when evidence supports it

Annotations should not come from the converter outputs used for the synthetic benchmark. Where possible, keep forms from the same lemma, passage, or speaker in the same evaluation split. Deduplicate exact matches or report them, and separate token-level from distinct-form results.

Along with candidate recall, report candidate precision or unsupported-candidate rate, candidate-set size, no-candidate rate, and results by process, dialect, and source where sample sizes allow. State how the evaluation handles multiple acceptable analyses.

## Next steps

1. Report both fixture-entry and distinct-input counts. Do not treat repeated rows as independent examples.
2. Link positive cases and negative controls to phonology notes or source passages.
3. Collect and annotate SEP–ACK/Atomic pairs from identified sources, preserving spelling, dialect, morphology, and ambiguity.
4. Measure over-generation as well as recall once accepted analyses are available.
5. Add independent sources and varieties. Keep evaluation examples separate from rule-development examples, and record provenance and licensing.