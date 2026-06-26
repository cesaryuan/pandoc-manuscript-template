# MathType TeX Input Probe Notes

This file records MathType TeX Input behavior that is easier to verify by
probing MathType or by inspecting saved MathType reference OLE files than by
reading the MTEF reference alone.

## Probe Commands

Use the local probe binary from `scripts/mathtype-rust`. To call MathType
through the helper and inspect fresh output:

```powershell
cargo run --bin probe_mathtype_tex -- --latex "$\xleftrightarrow{abc}$"
```

The helper path defaults to this repository's `MathTypeOleHelper.exe`. The
probe defaults to `--pre-verb 2`, matching the more stable helper README
recommendation; pass `--pre-verb 0` only when comparing against older runs.

Before using a fresh COM result as command-specific evidence, run a simple
helper baseline check:

```powershell
cargo run --bin probe_mathtype_tex -- --check-helper --timeout-ms 20000
```

If this `$x$` probe times out or produces an OLE file without `Equation
Native`, the MathType/COM session is unhealthy and command-specific probes
from the same run should not be used to classify TeX support.

When COM automation is unavailable, inspect an existing MathType reference OLE
or a raw MTEF file directly:

```powershell
cargo run --bin probe_mathtype_tex -- --ole samples\generated\mt_eq_015.ole.bin
cargo run --bin probe_mathtype_tex -- --mtef .pmt\debug-eq015-rust-norm.mtef.bin
```

The probe reads the `Equation Native` stream, skips the 28-byte native header,
and prints a compact MTEF record outline plus raw text fallback runs. Fresh
helper calls use a per-process subdirectory under `.pmt\probe-mathtype-tex`,
so several probes can run without sharing `probe.ole.bin`.

Fresh helper calls accept `--timeout-ms <N>` and default to 30000 ms. A timeout
means the local MathType COM/helper state did not produce output in time; it is
not by itself evidence that the TeX command is unsupported. Verify unsupported
behavior only from completed helper output or saved reference OLE files.
The probe tool removes its target `probe.ole.bin` before each helper call and
again after helper failure or timeout, because a killed COM helper can leave a
compound-file shell without an `Equation Native` stream.

## Unsupported Extensible Arrow

For:

```tex
$\xleftrightarrow{abc}$
```

MathType does not emit an extensible-arrow template. It stores the unsupported
control word as raw text CHAR records:

```text
raw_text_runs=\xleftrightarrow
```

Those raw text CHAR records use `options=0x80` and `typeface=fnTEXT`. The
grouped argument is then parsed normally, so `abc` appears as variable CHAR
records. The Rust writer mirrors this with `Expr::RawTex` plus the parsed group
content, and `samples/generated/eq_015.tex` captures this behavior.

The same raw fallback policy is kept for completed probes of unsupported
extensible-arrow variants such as `\xLeftrightarrow`,
`\xleftharpoonup`, `\xrightharpoonup`, `\xleftharpoondown`,
`\xrightharpoondown`, `\xleftrightharpoons`, and
`\xrightleftharpoons`. Other single-direction Supported Functions x-arrows
such as `\xLeftarrow`, `\xRightarrow`, `\xhookleftarrow`,
`\xhookrightarrow`, `\xtwoheadleftarrow`, `\xtwoheadrightarrow`, `\xmapsto`,
`\xlongequal`, and `\xtofrom` share the native `XArrowKind` template path.
Those are semantic native implementations based on the existing
`\xleftarrow`/`\xrightarrow` MathType template path, not fresh byte-level COM
probe claims for each glyph.

## Generated Table Probe Hygiene

The table generator writes one temporary `.tex` and `.ole.bin` pair per target
under `.pmt\mtef-table-generation`. Target names must be unique after ASCII
case folding, because Windows treats paths such as `special_gamma.ole.bin`
and `special_Gamma.ole.bin` as the same file. The generator checks this now;
uppercase Greek probes use names such as `special_uc_gamma` to avoid
overwriting lowercase probes.

To audit the saved probe cache without calling MathType COM, run:

```powershell
cargo run --bin generate_mtef_tables -- --report-existing
```

This reports extractable, missing, and invalid cached OLE targets. Missing
targets need a fresh MathType helper run; invalid targets usually mean the
selector changed or a stale cache file was produced before the case-folding
check existed.

Fresh table generation also accepts `--timeout-ms <N>`. If one MathType helper
probe hangs, the generator kills that helper process and removes the partial
OLE output instead of leaving a corrupt cache entry for later runs. Use
`--only <target-name-substring>` with `--report-existing` to inspect a focused
subset; filtered generation requires an explicit `--output` so the full
`char_tables.rs` is not accidentally replaced by a partial table.

To re-render `src\generated\char_tables.rs` from existing cache only, without
starting MathType COM, run:

```powershell
cargo run --bin generate_mtef_tables -- --existing-only --skip-invalid
```

This is useful after changing purely generated alias tables. Missing or invalid
probe families such as the current `mathfrak` targets are skipped instead of
blocking unrelated generated output.

The generator can also build an opt-in queue of single-command symbol probes
from `docs/Supported Functions.md`:

```powershell
cargo run --bin generate_mtef_tables -- --report-existing --supported-functions "docs\Supported Functions.md" --only supported_cmd_
```

This scans only symbol-like sections such as delimiters, letters, big
operators, logic, binary operators, relations, arrows, and
symbols/punctuation. The generated targets use `LastInferredChar`, so
MathType's own CHAR record supplies the
logical key instead of requiring a hand-written command-to-Unicode table first.
The queue is also pruned through the current parser and the shared
`raw_fallback.rs` evidence list: commands that already parse natively without
`Expr::RawTex`, or commands already known to be MathType raw fallback, are
skipped automatically. The queue is still a probe queue: unclassified raw
fallbacks and non-CHAR structures may remain until a real MathType run
classifies them. The Supported Functions extraction and pruning code lives in
`src/bin/generate_mtef_tables/supported.rs`, separate from the MTEF table
emission logic. In the current checkout this reports 0 doc-derived targets.

When `--supported-functions` is present, the report also prints
`supported_candidate_sections`, grouping those targets by their Supported
Functions section. The current queue is empty after semantic alias pruning. Use
`--supported-section <heading>` to focus that queue without a name-substring
hack, for example:

```powershell
cargo run --bin generate_mtef_tables -- --report-existing --supported-section Relations --only supported_cmd_
cargo run --bin generate_mtef_tables -- --report-existing --supported-section Arrows --only supported_cmd_
```

In the current checkout, those section-filtered reports contain 0 Relations
targets, 0 Negated Relations targets, 0 Arrows targets, 0 Delimiters targets,
and 0 Big Operators targets after parser/native/raw-fallback pruning.
Single-command Relations, Arrows, and Negated Relations queues are empty after
adding Unicode and short-sequence semantic aliases. A misspelled section name
fails instead of silently producing only the default table targets.

To materialize a reusable probe queue without invoking MathType, write a JSONL
manifest. `--supported-section` can be repeated to create one union queue:

```powershell
cargo run --bin generate_mtef_tables -- --supported-section Arrows --only supported_cmd_ --write-probe-manifest .pmt\probe-manifests\arrows.jsonl
cargo run --bin generate_mtef_tables -- --supported-section "Big Operators" --only supported_cmd_ --write-probe-manifest .pmt\probe-manifests\big-operators.jsonl
cargo run --bin generate_mtef_tables -- --supported-section Relations --supported-section "Negated Relations" --supported-section Arrows --only supported_cmd_ --write-probe-manifest .pmt\probe-manifests\relations-negated-arrows.jsonl
```

Each line records the target name, section, formula, expected temporary `.tex`
and `.ole.bin` paths, and whether the current cache is missing, invalid, or
already extractable. The manifest is useful when MathType COM is unstable:
prepare the queue now, then run or inspect the same targets later without
copying them from console output by hand. Manifest generation also writes each
target's `.tex` file under the configured work directory, so the paths in the
JSONL are directly usable by helper scripts. In the current checkout the
combined Relations/Negated Relations/Arrows manifest contains 0 missing probe
targets (0, 0, and 0 respectively).

For complete examples that are not single-symbol table targets, add
`--supported-snippets`. This mode is manifest-only: it parses Supported
Functions with the current parser, keeps examples that still contain raw TeX,
and removes commands already classified in `raw_fallback.rs`:

```powershell
cargo run --bin generate_mtef_tables -- --supported-functions "docs\Supported Functions.md" --supported-snippets --write-probe-manifest .pmt\probe-manifests\supported-unclassified-snippets.jsonl
cargo run --bin generate_mtef_tables -- --supported-section Units --supported-snippets --write-probe-manifest .pmt\probe-manifests\supported-units-snippets.jsonl
```

In the current checkout, the complete-formula unclassified queue is empty. If
future changes make it nonzero again, the all-section snippet manifest should
be treated as the next probe queue rather than as a list of parser failures.

Section-filtered generation is intentionally treated as a partial-table probe,
so it also requires an explicit output path instead of overwriting the main
generated table by default:

```powershell
cargo run --bin generate_mtef_tables -- --supported-section Arrows --only supported_cmd_curve --output .pmt\arrows-char-tables.rs --skip-invalid --timeout-ms 5000
```

Font table probes are also generated through `generate_mtef_tables.rs`.
`\mathcal`, `\mathbb`, and `\mathfrak` currently have extractable references
and therefore appear in `char_tables.rs`. `\mathfrak` uses a generated target
family with Windows-safe names (`mathfrak_uc_A` through `mathfrak_lc_z`):

```powershell
cargo run --bin generate_mtef_tables -- --report-existing --only mathfrak_
cargo run --bin generate_mtef_tables -- --only mathfrak_ --output .pmt\mathfrak-char-tables.rs
```

The 52 `\mathfrak` targets must be extracted from MathType output rather than
calculated from a single formula. For example, `\mathfrak{I}` uses
`FN_SYMBOL`, `mtcode=0x2111`, and `font_pos=0xc1`, while most letters use the
explicit Euclid Math One font slot.

Fresh `--pre-verb 2` probes show that legacy `\frak{...}` and KaTeX
`\mathsfit{...}` are not MathType TeX Input font commands. MathType stores the
control word as raw text and then parses the grouped letters normally, so these
commands stay in the known raw-fallback list instead of being mapped to
lookalike fonts.

To audit KaTeX/Supported-Functions coverage without calling MathType COM, run:

```powershell
cargo run --bin audit_supported_functions -- --limit 80
cargo run --bin audit_supported_functions -- --math-only --limit 20
cargo run --bin audit_supported_functions -- --math-only --write-unclassified-jsonl .pmt\probe-manifests\supported-math-unclassified.jsonl
cargo run --bin audit_supported_functions -- --math-only --write-remaining-jsonl .pmt\probe-manifests\supported-math-remaining.jsonl
```

The audit distinguishes parser coverage from writer coverage. `native_parse`
means the snippet parsed without raw TeX fallback; `native_render` means the
same AST also wrote MTEF bytes successfully. `raw_fallback` snippets are also
rendered, and `raw_render_error` should stay at zero so unsupported MathType
commands remain safely preserved as raw text. Treat `native_render` as the
stronger local support signal, and investigate any `render_error` before
counting a parsed command as implemented.
Use `--math-only` or `--view math` when focusing on complete formula behavior
instead of inline documentation fragments. `--write-unclassified-jsonl` writes
the complete-formula raw fallback queue with section names, snippets, and the
actual `Expr::RawTex` command names; this is the preferred input list for the
next MathType probe or native-implementation batch. `--write-remaining-jsonl`
adds a blocker bucket and short note for each still-unclassified formula, so
the queue stays useful even when MathType COM probing is temporarily unhealthy.

The audit now prints two corpora. `supported_functions_*` is the original
inline-code-span view, useful for command-level coverage. The
`supported_functions_math_*` view extracts complete `$...$` and `$$...$$`
formulas from the same document, which catches structural support for examples
that the code spans split into pieces, such as environments and nested
row-stack formulas. In the current checkout, the code-span view reports 862
native-rendered snippets, 82 raw-fallback snippets, zero unclassified raw
fallbacks, 100 syntax fragments, and zero parse/render errors. Many code-span
items are documentation fragments or placeholder syntax rather than complete
formulas. The complete-formula view reports 856 native-rendered formulas, 64
raw-fallback formulas, and zero parse, syntax, or render errors. All raw
fallbacks in both views are classified as known MathType raw text behavior.

Plain text and raw TeX fallback runs are emitted as UTF-16 code units rather
than Rust scalar-value casts. This preserves BMP output exactly and lets
non-BMP text examples such as mathematical alphanumeric characters write as
surrogate-pair CHAR records instead of failing local MTEF rendering.

Unsupported environments are preserved as raw TeX fallback instead of failing
parsing. A fresh `--pre-verb 2` probe of the KaTeX `CD` environment stores
`\begin` and `\end` as raw text runs and parses the interior as ordinary
characters; MathType does not emit a structural commutative-diagram object.

The parser supports a deliberately narrow `\def`/`\gdef` subset for Supported
Functions examples: zero-argument replacement (`\def\foo{x^2}`) and one
braced argument (`\gdef\foo#1{#1^2}`). Replacement text is parsed through the
same native parser path with a recursion limit. This is not a full TeX macro
engine; unsupported signatures remain raw fallback.

Array layout hints such as `\def\arraystretch{...}`, `\hline`, and
`\hdashline` are treated as non-visible layout controls. The matrix content is
kept native, but row/column rule styling is not yet reproduced in MTEF.

The audit also splits raw fallback into `known_mathtype_raw_fallback` and
`unclassified_raw_fallback`. Known raw commands are only those recorded in this
probe note as MathType raw text behavior. Raw command groups are derived from
the parser's `Expr::RawTex` nodes rather than the first control word in the
snippet, so examples like `\sum_{\mathclap{...}}` are grouped under
`\mathclap` instead of `\sum`. Unclassified raw commands are the next queue for
either native implementation, generated-table probes, or additional MathType
unsupported-behavior research.

For complete formulas, the audit also prints section and blocker counts. In
the current checkout, the unclassified raw-fallback queue is empty. Use any
future nonzero count to pick the next generated table, MTEF-template
implementation, or MathType-probe batch instead of hand-picking isolated
examples.

Modulo operators such as `\bmod`, `\mod`, `\pmod`, and `\pod` are parsed
natively as ordinary `mod` function text plus MathType spacing/parentheses.
A fresh helper probe for `\bmod` timed out in the current local COM state, so
this is a semantic native implementation rather than a byte-level MathType
reference claim.

The Supported Functions math-operator list includes `\ch` as a standalone
function name, so it uses the same native `FunctionName` path as `\ctg`,
`\sinh`, and similar operator aliases.

Spacing aliases such as `\thinspace`, `\negthinspace`, `\<space>`, `\space`,
and `\nobreakspace` reuse the same `Expr::Space` records already used for
`\,` and `\!`. Fresh `--pre-verb 2` probes show `\medspace` writes a native
`fnSPACE` CHAR with width byte `0x02`, `\thickspace` writes width `0x04`, and
both `\negmedspace` and `\negthickspace` write the same negative-space width
as `\!` (`0x01`). MathType stores `\enspace` as raw text, so it remains known
raw fallback instead of being mapped to a guessed spacing width.

MathType accepts `\hspace{...}` without emitting either raw text or a visible
spacing CHAR in the probed examples, so the parser treats it as an ignored TeX
layout hint. Array-only hints such as `\cline{1-2}` are handled the same way:
the span argument is consumed to keep the surrounding table native, but no
visible rule is written.

TeX style switches `\displaystyle`, `\textstyle`, `\scriptstyle`, and
`\scriptscriptstyle` are preserved in the AST as `Expr::Style` and written with
MathType's documented logical-size records: display/text use Full size, script
uses Sub size, and scriptscript uses Sub2 size. This is a semantic writer
implementation based on the MTEF size-record spec, not a fresh COM probe.

`\underset{...}{...}` is parsed natively as the lower-slot counterpart to the
existing `\stackrel`/`\overset` support. It uses the documented `tmLIM`
template selector (`0x17`) and the lower-only variation already used by
`\lim_{...}` (`0x10`), with an empty upper slot.

`\vcenter{...}` is treated as a semantic content wrapper for math content.
`Supported Functions.md` documents that it can wrap math directly when strict
rendering is disabled, and MathType's fenced templates already center their
visible slot content. This removes raw fallback for the direct-math form
without claiming support for TeX primitives such as `\hbox`.

TeX font-size switches are different from those math-style switches. Saved OLE
probes for `$\Large AB$` (`.pmt\probe-large\run-24924\probe.ole.bin`) and
`$\Huge AB$` (`.pmt\probe-huge\run-48544\probe.ole.bin`) contain raw text runs
for `\Large` and `\Huge` (`CHAR` records with `options=0x80`), while the
following `AB` remains ordinary parsed math characters. The current COM helper
timed out on a fresh `$\small AB$` probe, so the size-switch family is recorded
as known MathType raw fallback instead of being guessed as native `SIZE`
records.

`\pmb{...}` is parsed as a bold-font wrapper and reuses the existing MathType
vector/bold CHAR path. Bare `\pmb` is intentionally left as raw fallback
because it is an incomplete command, not a renderable font expression.

`\mathtt{...}` and `\texttt{...}` use MathType's fixed `fnUSER1` style. The
fixed MTEF definition block maps that style to Courier New, so these commands
do not need a new dynamic FONT_DEF record.

`\verb<delimiter>...<delimiter>` is parsed as literal visible text in the same
fixed-width `fnUSER1` style. This is a semantic implementation for the
delimiter form: characters inside the delimiters are emitted literally, so
syntax characters such as `^` do not become scripts.

`\textrm{...}` is a roman-font alias, and `\tt ...` reuses the same Courier New
path as `\mathtt{...}`. Function-name aliases such as `\sinh`, `\tanh`, and
`\tg` reuse the ordinary MathType function-text writer.

KaTeX-only HTML wrappers (`\htmlId`, `\htmlClass`, `\htmlStyle`, and
`\htmlData`) drop their web attributes and keep the wrapped math content.
Overlap/layout wrappers (`\mathclap`, `\mathllap`, `\mathrlap`, and `\smash`)
also keep their visible content while ignoring the width/height hint. These are
semantic content-preserving fallbacks, not claims that MathType receives the
same HTML attributes or TeX box metrics.

Box-color commands (`\colorbox{...}{...}` and `\fcolorbox{...}{...}{...}`)
also keep the visible math content and ignore the background/frame colors. This
matches the current content-preserving wrapper policy; it is not full native
colored-box layout support.

`\raisebox{...}{...}` follows the same policy: the lift/height/depth metrics are
TeX box layout hints, so the writer keeps the visible math content and ignores
the vertical offset instead of emitting raw fallback.

Fresh `--pre-verb 2` probes classify `\r{a}`, `\mathstrut`, `\phantom`,
`\hphantom`, `\vphantom`, `\kern`, `\mathrel`, `\nonumber`, `\notag`, and
`\hbox` as MathType raw text behavior. The writer therefore preserves them as
known raw fallback unless a separate native, visible-content-preserving policy
has already been implemented for a higher-level wrapper.

Fresh `--pre-verb 2` probes for `\rule{10pt}{10pt}` and the related unit
examples store `\rule` as raw text (`options=0x80`) followed by the dimension
tokens as ordinary parsed characters. The writer therefore preserves `\rule`
as known MathType raw fallback instead of approximating it with a box template.

KaTeX renders `\phase{...}` as MathML `menclose` with `phasorangle` notation.
The current MTEF writer has no dedicated phasor-angle enclosure template, so
the parser keeps the visible phase-angle meaning by emitting `∠` followed by
the parsed argument.

`\not` consumes the following relation atom and maps it through a small Unicode
negated-relation table, so examples such as `\not =`, `\not\in`, and
`\not\Rightarrow` render as native relation characters. If the operand is not a
known relation atom, the parser preserves the raw `\not` prefix and keeps the
operand visible instead of guessing.

`\widecheck{...}` reuses the existing explicit check-accent stack template. It
is treated as the same accent family as `\check{...}` rather than a separate
MathType-specific probe result.

`\char"263a`-style hexadecimal Unicode escapes are parsed as real Unicode
characters. Bare `\char` is intentionally left as raw fallback and skipped from
the generated single-command probe queue because it is an incomplete command,
not a meaningful MathType symbol target.

`\vert` is a generated delimiter alias for `|`. `\middle` is different: it is
a structural command that requires a following delimiter, so complete forms
such as `\left(x\middle|y\right)` parse natively, while bare `\middle` stays
raw fallback and is skipped from the single-command probe queue.

Single-command arrow, relation, delimiter, letterlike, big-symbol, and
punctuation aliases such as `\curvearrowleft`, `\leftrightharpoons`,
`\longmapsto`, `\leqq`, `\gnapprox`, `\varsubsetneqq`, `\bigvee`, `\daleth`,
`\lmoustache`, and `\varDelta` are generated into `TEX_COMMAND_CHARS` from the
Supported-Functions-oriented Unicode alias table in
`src/bin/generate_mtef_tables/symbol_aliases.rs`. Text-style aliases such as
`\copyright`, `\pounds`, `\mathsterling`, `\yen`, `\lq`, and `\rq` are
generated into `TEX_COMMAND_TEXTS` and written through `Expr::Text` so they
use MathType text CHAR records. They are semantic native support: the parser
emits the matching Unicode symbol or text and the writer uses MathType's
symbol/text style paths. This is separate from MathType COM byte-level probing,
which can still be used later for commands that require source-command-specific
encoding.

The `\image` alias maps to the same U+2111 glyph as `\Im`, matching KaTeX's
Supported Functions table. It is generated as a normal command alias rather
than as a command-specific hard-coded branch.

For colon-composed relations, aliases with a single Unicode relation in
KaTeX's macro source are mapped semantically, such as `\dblcolon` to U+2237,
`\coloneqq` and `\colonequals` to U+2254, `\eqqcolon` and `\equalscolon` to
U+2255, and `\eqcolon` and `\minuscolon` to U+2239. Short visible expansions
such as `\coloneq`, `\colonminus`, `\colonapprox`, `\colonsim`, `\ratio`,
`\coloncolonequals`, `\equalscoloncolon`, `\coloncolonminus`,
`\minuscoloncolon`, `\coloncolonapprox`, `\coloncolonsim`,
`\simcoloncolon`, and `\approxcoloncolon` are generated into
`TEX_COMMAND_SEQUENCES` instead of being special-cased in the parser. A local
probe attempt for representative commands including `\coloneqq`, `\dblcolon`,
`\ratio`, and `\approxcoloncolon` timed out in the current COM state; timeout
is not evidence of unsupported input, so these sequence aliases are semantic
native support rather than byte-level MathType reference claims.

Bra-ket and set macros such as `\ket`, `\braket`, and `\Set` are also parsed
to native delimiter structures where the existing writer has a matching
template. `\bra` is currently written as ordinary visible angle/content/bar
characters instead of guessing MathType's scalable Dirac-bra template bytes.

## Uppercase Greek Command Probes

MathType TeX Input accepts most KaTeX-style uppercase Greek control words as
native uppercase Greek CHAR records, including `\Alpha`, `\Beta`, `\Chi`,
`\Epsilon`, `\Eta`, `\Iota`, `\Kappa`, `\Mu`, `\Nu`, `\Rho`, `\Tau`, and
`\Zeta`. These are included in the generated special-character table.

`\Omicron` is different: MathType stores the literal command as raw text CHAR
records with `options=0x80` and `typeface=fnTEXT`, just like unsupported
extensible arrows. Keep it on the raw fallback path rather than mapping it to
Latin `O` or Greek omicron by assumption.

Lowercase `\omicron` is semantically supported as Greek omicron U+03BF. A local
helper probe for `\omicron` timed out, so this is not a fresh byte-level COM
claim. The writer derives its CHAR record from MathType's documented lowercase
Greek typeface convention and the Symbol-font slot `o`.

## Symbol Alias Probes

MathType accepts several KaTeX-style aliases as native CHAR records, including
`\Box`, `\Diamond`, `\Longleftarrow`, `\Longrightarrow`,
`\Longleftrightarrow`, `\Lsh`, `\Rsh`, `\Vert`, `\Uarr`, `\alef`, `\P`,
`\S`, `\infin`, `\checkmark`, `\sdot`, `\cdots`, `\vdots`, `\ddots`, and
`\dotsb`. Additional one-character commands such as `\amalg`, `\approxeq`,
`\backepsilon`, `\backprime`, `\backsim`, `\backsimeq`, `\barwedge`,
`\between`, `\bigstar`, `\bigtriangleup`, `\bigtriangledown`,
`\blacksquare`, `\boxdot`, `\boxminus`, `\boxplus`, `\boxtimes`, `\bumpeq`,
`\circlearrowleft`, `\circlearrowright`, `\circledS`, `\alefsym`,
`\circledast`, `\circleddash`, `\circledcirc`, `\complement`, `\curlyvee`,
`\curlywedge`, `\divideontimes`, `\doublebarwedge`, `\intercal`,
`\leftthreetimes`, `\lessdot`, `\ltimes`, `\rightthreetimes`, `\rtimes`, and
`\smallsetminus` also probe as native CHAR records. These are generated from
probe targets and covered by generated reference samples rather than by
handwritten typeface values.

`\sdot` is not encoded like `\cdot`: MathType stores it as U+00B7 middle dot
with no font position, while `\cdot` uses U+22C5 with a Symbol font position.
Keep those logical characters separate even though they look similar.

Commands such as `\Join`, `\Colonapprox`, `\Coloneq`, `\Coloneqq`,
`\Colonsim`, `\Eqcolon`, `\Eqqcolon`, `\approxcolon`, `\simcolon`,
`\vcentcolon`, `\TeX`, `\LaTeX`, `\KaTeX`, `\dag`, `\ddag`, `\cdotp`,
`\allowbreak`, `\circledR`, `\dotminus`, `\dotsc`, `\dotsi`, `\dotsm`, and
`\dotso` currently probe as raw text fallback in MathType TeX Input, so the
parser should not map them to Unicode lookalikes.

`\centerdot` probes as native but uses a different CHAR encoding from `\cdot`
for the same logical U+22C5 symbol. The parser keeps it as
`Expr::CommandSymbol`, and the writer uses `COMMAND_SPECIFIC_CHARS` instead of
the ordinary Unicode-to-CHAR table.

## Single-Character Accent Probes

MathType writes `\breve{a}`, `\dot{a}`, `\ddot{a}`, `\dddot{a}`,
`\ddddot{a}`, and `\tilde{a}` as one CHAR record with an attached EMBELL
record. The observed EMBELL subtypes are `0x14`, `0x02`, `0x03`, `0x04`,
`0x18`, and `0x08`, respectively. The writer uses the same `Expr::Accent`
path as `\bar` and `\hat` for these single-character cases.

`\acute{...}`, `\grave{...}`, and `\check{...}` are also native in MathType,
but they use the same stacked template selector as `\stackrel`
(`selector=0x17`, `variation=0x20`) with a separate visible accent glyph in
the upper slot (`0x00b4`, `0x0060`, or `0x02c7`). The writer handles these as
explicit accent templates rather than EMBELL records.

Alias commands `\u{...}` and `\v{...}` reuse the existing `\breve{...}` and
`\check{...}` native writer paths. `\H{a}` probes as raw text fallback in
MathType TeX Input: the saved OLE file
`.pmt\probe-mathtype-tex\run-64080\probe.ole.bin` stores `\H` as raw text
`CHAR` records (`options=0x80`) and then writes `a` as ordinary parsed math.
The ring alias `\r{...}` remains unclassified until the writer has a native
ring template instead of approximating it.

`\mathring{g}` and `\widetilde{ac}` probe as raw text fallback in MathType TeX
Input.

The MTEF specification also defines `embU_TILDE` (`0x1e`) for an under-tilde
character embellishment. The parser therefore maps simple character runs such
as `\utilde{AB}` to native per-character embellishments. More complex
`\utilde{...}` arguments stay raw until a template-level MathType mapping is
available.

## Arrow Accent Templates

MathType stores `\vec`, `\overrightarrow`, and related over/under arrow accents
with the `tmVEC` template (`selector=0x1f`). The writer now uses a dedicated
`Expr::ArrowAccent` path so direction, under-placement, and harpoon state are
derived from template variation bits instead of being special-cased as
`\overrightarrow` only. The trailing expanding glyph follows the same pattern
as the existing right-arrow path (`U+20D7`), using the corresponding Unicode
combining arrow mark for left, left-right, under, and harpoon variants.

## Text Mode Parsing

`\text{...}` is parsed through `parser/text_mode.rs` instead of being stored as
an opaque `Text(String)`. Known text commands such as `\textdegree`, `\OE`,
`\P`, and `\textcircled a` are expanded to literal text characters, while
`\sout{...}` becomes the same native overstrike template used in math mode.
Text-mode accent commands such as `\'{a}`, `\"{a}`, and `\H{a}` are emitted as
plain text plus Unicode combining marks when their argument is itself plain
text; complex arguments stay on the raw fallback path. This keeps accent
support systematic without maintaining a large precomposed-character table.
Unknown text-mode commands intentionally become `Expr::RawTex` so the Supported
Functions audit does not count unimplemented TeX syntax as native support just
because it appeared inside a text run.

## Large Operator Probes

`\sum`, `\prod`, `\coprod`, `\bigcup`, and `\bigcap` use MathType's large
operator template with selectors `0x10` through `0x14`. The glyph appended by
that template is still a normal CHAR record, but its `typeface`, `mtcode`, and
`font_pos` vary by command. The generator therefore extracts all three fields
into `BIG_OPERATOR_GLYPHS` instead of assuming `fnSYMBOL`.

Other visually similar commands are not necessarily the same template family.
Probes of `\bigoplus`, `\bigotimes`, `\bigodot`, and `\biguplus` show an
ordinary glyph with script templates, plus a `0x0d` size marker before each
scripted big symbol. The generator verifies these command aliases in
`BIG_SYMBOL_COMMAND_CHARS`, and the writer keeps their source-command identity
with `Expr::BigSymbol` so ordinary `\oplus` remains unchanged.

`\bigsqcup` is similar visually but uses MathType's `tmSUMOP` template selector
`0x16` with variation `0x70` when both lower and upper scripts are present,
followed by an explicit Euclid glyph slot. The generator verifies it in
`SUM_OPERATOR_COMMAND_CHARS`, and the writer preserves it as
`Expr::SumOperatorSymbol` instead of replaying a captured body.

Standalone large operators with limits, such as `\sum_{i=1}^n`, use the same
large-operator template path in the writer with an empty first slot. This keeps
the local MTEF writer structurally complete for Supported Functions audits, but
the exact byte shape should be promoted to a generated reference sample when
fresh MathType COM probing is stable again.

## Delimiter Command Aliases

Delimiter spellings such as `\langle`, `\rangle`, `\lbrace`, `\rbrace`,
`\lbrack`, `\rbrack`, `\lceil`, `\rfloor`, `\lVert`, and `\rVert` are kept in
the generated `DELIMITER_COMMAND_CHARS` table. The parser uses the same table
both after `\left`/`\right` and when those commands appear as standalone
visible delimiters. This keeps delimiter spelling aliases centralized rather
than scattering parser-only one-offs.

Supported Functions examples for `\Set` and `\Braket` display `\VERT`, while
their code spans use `\|`. The parser maps `\VERT` to the same double-vertical
bar character as `\Vert`/`\|`, so those examples no longer need a raw fallback
only because the display source and code-span source differ.

## Horizontal Fence Templates

`\overbrace` and `\underbrace` use MathType's `tmHBRACE` selector (`0x18`),
with variation `0x01` for top and `0x00` for bottom. `\overbracket` and
`\underbracket` use the sibling `tmHBRACK` selector (`0x19`) with the same
top/bottom variation bit. Both AST variants share the same optional annotation
slot handling, so `\overbracket{a+b}^{\text{note}}` and
`\underbracket{a+b}_{\text{note}}` stay native instead of becoming scripted
raw TeX fallback.

The MTEF template table does not list separate selectors for KaTeX's
`\overgroup`, `\undergroup`, `\overlinesegment`, or `\underlinesegment`.
Fresh `--pre-verb 2` probes show MathType stores each of those control words
as raw text and then parses the grouped letters normally. Keep those commands
on the known raw fallback path rather than mapping them to brace/bracket
lookalikes.

## Annotation Probes

Fresh `--pre-verb 2` probes for `\tag{hi}` and `\tag*{hi}` store `\tag` as raw
text and then parse the argument/body as ordinary visible math. The writer
keeps this MathType behavior instead of inventing equation-numbering semantics.

## TeX Infix Fractions

Old TeX infix commands are handled at sequence level rather than as ordinary
prefix commands. The parser maps `{a \over b}` to the same native fraction AST
as `\frac{a}{b}`. It maps `{a \atop b}`, `{n \choose k}`, `{n \brace k}`, and
`{n \brack k}` to a shared two-row pile AST whose writer selects no delimiters,
parentheses, braces, or brackets. `\above{...}` now shares the fraction path,
and `\genfrac` maps to the existing fraction/pile plus optional delimiter
templates. The line-thickness and style arguments are parsed but not yet
reproduced as distinct MTEF size/line records.

`\bigvee` and `\bigwedge` also need separate treatment because probes use
expandable glyph records whose `mtcode` can be `0xfffd`. Do not route these
through the large-operator template by assumption.

## Sized Delimiter Probes

`\big( x \big)` and `\Bigl[ y \Bigr]` are accepted by MathType without raw text,
but they do not simply erase the size commands in every context. A combined
sample showed byte-level MTEF differences when the Rust parser treated the
size commands as ordinary delimiters. `\bigm|` is stored as raw text fallback.

Keep `\big`, `\Big`, `\bigl`, `\Bigr`, `\bigm`, and related size commands on
the raw fallback path until the delimiter-size record shape is implemented
explicitly.

## Probe Timeout Hygiene

Helper calls that use `--pre-verb 0` can time out even for a known simple
formula such as:

```powershell
cargo run --bin probe_mathtype_tex -- --latex "\alpha+x" --pre-verb 0 --timeout-ms 5000
```

Use `--pre-verb 2` for fresh MathType TeX Input probes and table generation.
The Rust probe and generator default to `2` so completed helper output can be
used as command-specific evidence. A timeout is still not evidence of
unsupported TeX; classify unsupported behavior only from completed helper
output or saved reference OLE files.
