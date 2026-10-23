**Phonology Reference for SEP Reconstruction**

This project reference summarizes processes in Edward H. Trager’s notes on Huasteca Nahuatl from the Chicontepec area. It focuses on processes used in the repository’s M→F conversion and SEP reconstruction. The notes describe one variety; these processes may differ across Nahuatl varieties.

**Orthography classes (M vs. F)**

Trager distinguishes morphophonemic (M) orthographies from phonetic (F) orthographies:

- **M:** Andrews-Campbell-Karttunen (ACK) and Trager’s abugida. These preserve underlying contrasts and morphological information.
- **F:** SEP and Hasler. These represent surface pronunciations and allophonic changes.

For example, the notes contrast ACK `quicouhqui` with SEP `kikojki`, and ACK `nosin` with SEP `nosij`. These are not only spelling substitutions: an F form may not contain enough information to recover a single M form.

In this repository, `apply_m2f()` applies forward allophonic changes to produce SEP- or Hasler-like forms. `reconstruct_sep()` proposes underlying Atomic candidates from SEP rather than selecting one M form. Its projection check tests compatibility with the implemented forward mapping; it does not prove a linguistic analysis.

**/w/ allophony**

The notes describe /w/ as [w] at the beginning of a syllable and [h] at the end of a syllable. The examples contrast ACK `quicohua` / SEP `kikoua` for onset /w/ with ACK `quicouhqui` / SEP `kikojki` for coda /w/.

`apply_m2f()` applies the forward coda-w-to-h rule through `w2h()`. `CodaWRecovery` proposes underlying `w` for surface `h` in the contexts used by `reconstruct_sep()`. The notes support the phonological motivation, but each candidate needs context-sensitive evaluation because surface `h` can have other sources.

**/kʷ/ alternations**

According to the notes, /kʷ/ is [kʷ] at syllable onset, [k] in coda position, and [h] in coda position before a following /k/. Examples include ACK `cueponi` / SEP `kueponi` for onset /kʷ/, ACK `neuctli` / SEP `nektli` for coda [k], and ACK `tzaucqui` / SEP `tsajki` for coda [h] before /k/.

`apply_m2f()` performs the forward alternations in `kw2k_or_h()`. `KwRecovery` proposes underlying Atomic `κ` where the implemented contexts allow a coda [k] or [h] outcome. This recovery must preserve ambiguity: the notes caution that surface [k] does not establish underlying /kʷ/. Context and lexical evidence may be needed to assess a candidate.

**Final /n/ realization**

For the Chicontepec variety, the notes describe word-final /n/ as weakening, in many cases to a light aspiration represented as [h]. The example contrasts ACK `nosin` with SEP `nosij` (“my corn”). The notes also identify contexts where final /n/ may remain represented as `n`, including some monosyllabic forms such as `tlen` and `tlan`; the alternation is not unconditional.

`apply_m2f()` applies the forward final-n-to-h change in `n2h()`. `FinalNRecovery` proposes final `n` when a normalized input ends in `h`, while keeping the surface-shaped candidate as an alternative. This inverse generates hypotheses; it is not a rule for replacing every final `h` with `n`.

**Coda /k/ lenition**

The notes describe coda /k/ as becoming [h] (or a weaker aspiration) before a following syllable that begins with a consonant. The examples contrast ACK `nicnequi` / SEP `nijneki` before a consonant with ACK `niquihtoa` / SEP `nikijtoa` before a vowel.

`apply_m2f()` applies the forward change through `k_cons_to_h_cons()`. `KLenitionRecovery` proposes `k` from surface `h` in a bounded context intended to match that forward process, then uses projection validation. Because /w/ and /kʷ/ can also yield [h], an `h` in SEP alone does not establish underlying /k/.

**/ll/ and /hl/**

The notes’ M→F example includes ACK `niquillamiqui` and SEP `nikijlamiki`, with the corresponding Atomic and allophonic forms shown in the NOCE output. This illustrates the `ll` to `hl` behavior in the implementation’s `ill[a|i]` environment.

`apply_m2f()` applies this change through `ll2hl()`. `LlHlRecovery` proposes `ll` only in the inverse context represented by that forward rule. This rule is separate from the /w/, /k/, /kʷ/, and final /n/ alternations, though multiple changes can occur in one word.

**Implications for reconstruction**

The documented M→F rules explain why a shallow SEP spelling can correspond to several underlying Atomic forms. The reconstruction module generates candidates and preserves ambiguity rather than selecting one:

- **Coda /w/ → [h]:** `w2h()` → `CodaWRecovery`
- **Coda /kʷ/ → [k] or [h]:** `kw2k_or_h()` → `KwRecovery`
- **Word-final /n/ → [h]:** `n2h()` → `FinalNRecovery`
- **Coda /k/ → [h] before a consonant:** `k_cons_to_h_cons()` → `KLenitionRecovery`
- **/ll/ → [hl] in a restricted environment:** `ll2hl()` → `LlHlRecovery`

For each candidate, `reconstruct_sep()` uses the forward projection check and removes duplicate Atomic forms. These checks establish compatibility with the implementation; they do not establish that a candidate is attested, lexically correct, or suitable for every dialect. Evaluation on attested SEP forms paired with reviewed ACK/Atomic analyses is needed to assess recall and unsupported alternatives.

**References**

- Trager, Edward H. “Notas sobre la fonología del náhuatl de la Huasteca cerca de Chicontepec, ortografías usadas para escribir el lenguaje, y implicaciones en la app Yoltok.” 2021-12-21; updated 2022-02-22. [PDF in the NOCE repository](https://github.com/piratical/noce/blob/main/documentation/notas_sobre_la_fonologia.pdf). Relevant sections: 3.1 and 4.1–4.4.
- [NOCE (Nahuatl Orthography Conversion Engine) repository](https://github.com/piratical/noce), including the source notes and conversion implementation.
