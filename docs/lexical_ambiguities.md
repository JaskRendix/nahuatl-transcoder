# Lexical ambiguity and ranking notes

The repository's reconstruction diagnostics were run against `g2.nfc.txt`. The current run processes 6,033 headwords; results below describe this bundled lexicon and the project's own conversion and reconstruction code.

* **candidate coverage:** 100% (6,033/6,033 input-derived Atomic forms found among candidates)
* **misses:** 0
* **original form recovered:** 100%
* **original was the sole lexicon hit:** 99.80% (6,021/6,033)

The 12 headword entries with multiple lexicon hits form six pairs that share a SEP output in this run. These are observed collisions under the implemented forward mapping; the result is specific to this lexicon and does not establish that every candidate is linguistically valid.

---

## Ambiguity types

### Final `h` vs. final `n`

SEP collapses `-h` and `-n` into `j`.

* `lah` / `lan` $\rightarrow$ `laj`
* `tlah` / `tlan` $\rightarrow$ `tlaj`
* `tleh` / `tlen` $\rightarrow$ `tlej`

Reconstructing any of these outputs yields both forms as candidates; both occur as entries in the bundled lexicon.

### Final `w` vs. final `n`

SEP maps both `-w` (`-uh`) and `-n` to `j` in certain environments.

* `tiuh` / `tin` $\rightarrow$ `tij` (reconstructs to `tiw` / `tin`)

### Coda-`k` lenition

forward phonology neutralizes coda `k` against `h`.

* `quechcuayohtli` / `quechcuayoctli` $\rightarrow$ `kechkuayojtli`

### Coda `w`

The implemented M→F rule changes coda `w` to `h`, which the SEP renderer writes as `j` in the example below.

* `cuahyōlli` / `cuauhyōlli` $\rightarrow$ `kuajyōlli`

---

## The 12 ambiguous entries

```text
cuahyōlli      <-> cuauhyōlli
lah            <-> lan
quechcuayohtli <-> quechcuayoctli
tiuh           <-> tin
tlah           <-> tlan
tleh           <-> tlen
```

Each pair accounts for two lexicon entries, for 12 entries with non-unique lexicon hits.

---

## takeaway

For these listed headwords, the diagnostic finds both lexicon forms as candidates for the shared SEP output. This illustrates information loss in the implemented forward mapping:

```text
ACK / Atomic -> SEP -> Reconstruction
```

SEP may discard distinctions, so reverse mapping alone need not identify a unique underlying form. This observation applies to the listed examples; it does not establish completeness of candidate generation for all forms.

---

## Ranking performance

Using the current derivation-based scoring heuristic on the full `g2.nfc.txt` lexicon:

```text
Processed entries: 6033

Top-1 accuracy: 5343 / 6033 (88.56%)
Top-3 accuracy: 5929 / 6033 (98.28%)

Mean rank: 1.202
Worst rank: 22
```

## Rank distribution

```text
Rank 1 : 5343 (88.56%)
Rank 2 :  486 ( 8.06%)
Rank 3 :  100 ( 1.66%)
Rank 4+:  104 ( 1.72%)
```

## interpretation

For this bundled lexicon run, the reconstruction engine achieves:

```text
Coverage: 100%
Misses:   0
```

The current heuristic places the input-derived Atomic form:

```text
1st place 88.56% of the time
Top 3     98.28% of the time
```

These are lexicon-run ranking results, not accuracy estimates for independent SEP data:

```text
Correct candidate missing          : 0.00%
Correct candidate not ranked #1    : 11.44%
```

They do not establish that the candidate space is complete or that the highest-ranked form is the most likely linguistic analysis. Independent evaluation is needed to measure both missed analyses and unsupported candidates.

---

## Possible future work

* frequency weights
* morphological parsing
* context awareness
* ML disambiguation

These may help rank candidates, but the observed SEP collisions cannot be resolved from the SEP form alone.

---

# Historical progress notes

The following progression is retained from earlier documented benchmark runs:

Early runs on `g2.nfc.txt` missed entries. The documented miss count decreased in these steps:

```text
87 -> 69 -> 28 -> 13 -> 3 -> 0
```

---

## Documented changes during that progression

* **labialized affricates:** fixed `chua`, `chue`, and `chui` sequences
* **rule composition:** stacked `FinalNRecovery` with `CodaWRecovery`
* **targeted coda-w:** added position-specific coda `w` recovery
* **k-lenition stack:** combined `KLenitionRecovery` with:
* `FinalWRecovery`
* `CodaWRecovery`
* `LlHlRecovery`

---

## Recorded end result

```text
Recovered entries: 6033 / 6033
Misses:            0
Coverage:          100.00%
```

These historical counts describe intermediate runs, not independently reproducible results from this repository. The current diagnostic run reports the lexicon figures above; neither these figures nor the historical progression establish complete coverage of naturally occurring SEP forms.

---

## Statistics summary

```text
Lexicon entries:           6033
Coverage:                  6033 / 6033 (100.00%)
Misses:                    0

Original recovered:        6033 / 6033 (100.00%)
Original sole hit:         6021 / 6033 (99.80%)
Ambiguous entries:         12

Top-1 ranking accuracy:    5343 / 6033 (88.56%)
Top-3 ranking accuracy:    5929 / 6033 (98.28%)

Mean rank:                 1.202
Worst rank:                22
```

The figures above are specific to the bundled lexicon and current implementation. They do not show that every remaining ambiguity in other data is a collision, or that reconstruction is complete for all Nahuatl varieties and sources.
