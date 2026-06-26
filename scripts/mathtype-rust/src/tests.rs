use crate::ast::{EnvironmentKind, Expr, MatrixKind};
use crate::cfb::read_regular_stream;
use crate::mtef::write_mtef;
use crate::parser::{normalize_latex, Parser};
use crate::typeface::{
    EXPLICIT_FONT_NEG_1, FN_EXPAND, FN_FUNCTION, FN_LC_GREEK, FN_MT_EXTRA, FN_SPACE, FN_SYMBOL,
    FN_TEXT, FN_TEXT_FE, FN_USER1,
};

use std::fs;
use std::path::{Path, PathBuf};

/// Compare every sample under samples/ with its MathType reference output.
#[test]
fn all_samples_match_mathtype_mtef() {
    let sample_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("samples");
    let mut cases = sample_paths(&sample_root).expect("failed to enumerate samples");
    assert!(
        !cases.is_empty(),
        "no samples found in {}",
        sample_root.display()
    );

    let sample_count = cases.len();
    let mut failures = Vec::new();
    for tex_path in cases.drain(..) {
        let stem = tex_path
            .file_stem()
            .and_then(|value| value.to_str())
            .expect("sample file stem is valid UTF-8");
        let number = stem
            .strip_prefix("eq_")
            .expect("sample file name starts with eq_");
        let mt_path = tex_path
            .parent()
            .expect("sample file has a parent directory")
            .join(format!("mt_eq_{number}.ole.bin"));

        match compare_sample(&tex_path, &mt_path) {
            Ok(()) => {}
            Err(err) => failures.push(format!("{}: {err}", tex_path.display())),
        }
    }

    println!(
        "MTEF comparison samples: {}/{} passed",
        sample_count - failures.len(),
        sample_count
    );
    assert!(
        failures.is_empty(),
        "MTEF comparison failed for {} sample(s):\n{}",
        failures.len(),
        failures.join("\n")
    );
}

/// Return all sample TeX files under the samples tree in stable order.
fn sample_paths(sample_root: &Path) -> Result<Vec<PathBuf>, String> {
    let mut paths = Vec::new();
    collect_sample_paths(sample_root, &mut paths)?;
    paths.sort();
    Ok(paths)
}

/// Recursively collect eq_*.tex files so every sample subdirectory is tested.
fn collect_sample_paths(dir: &Path, paths: &mut Vec<PathBuf>) -> Result<(), String> {
    for entry in
        fs::read_dir(dir).map_err(|err| format!("failed to read {}: {err}", dir.display()))?
    {
        let path = entry
            .map_err(|err| format!("failed to read entry in {}: {err}", dir.display()))?
            .path();
        if path.is_dir() {
            collect_sample_paths(&path, paths)?;
        } else if path
            .file_name()
            .and_then(|value| value.to_str())
            .is_some_and(|name| name.starts_with("eq_") && name.ends_with(".tex"))
        {
            paths.push(path);
        }
    }
    Ok(())
}

/// Generate Rust MTEF for one sample and compare it with MathType's reference OLE stream.
fn compare_sample(tex_path: &Path, mt_path: &Path) -> Result<(), String> {
    let raw_latex = fs::read_to_string(tex_path)
        .map_err(|err| format!("failed to read {}: {err}", tex_path.display()))?;
    let latex = normalize_latex(&raw_latex);
    let rust_mtef = render_mtef_for_test(&latex)?;
    let mt_ole = fs::read(mt_path)
        .map_err(|err| format!("failed to read reference {}: {err}", mt_path.display()))?;
    let equation_native = read_regular_stream(&mt_ole, "Equation Native")?;
    if equation_native.len() < 28 {
        return Err("Equation Native stream is shorter than the native header".to_string());
    }
    let mt_mtef = &equation_native[28..];

    if rust_mtef == mt_mtef {
        return Ok(());
    }

    let first_diff = rust_mtef
        .iter()
        .zip(mt_mtef.iter())
        .position(|(left, right)| left != right)
        .unwrap_or_else(|| rust_mtef.len().min(mt_mtef.len()));
    Err(format!(
        "MTEF differs: mathtype_len={}, rust_len={}, first_diff={first_diff}, tex={latex}",
        mt_mtef.len(),
        rust_mtef.len()
    ))
}

/// Ensure common matrix environments take the native MATRIX path, not raw TeX fallback.
#[test]
fn matrix_environments_parse_and_render_natively() {
    let cases = [
        ("\\begin{matrix}a&b\\\\c&d\\end{matrix}", MatrixKind::Plain),
        (
            "\\begin{smallmatrix}a&b\\\\c&d\\end{smallmatrix}",
            MatrixKind::Plain,
        ),
        (
            "\\begin{pmatrix}a&b\\\\c&d\\end{pmatrix}",
            MatrixKind::Parenthesized,
        ),
        (
            "\\begin{bmatrix}a&b\\\\c&d\\end{bmatrix}",
            MatrixKind::Bracketed,
        ),
        (
            "\\begin{Bmatrix}a&b\\\\c&d\\end{Bmatrix}",
            MatrixKind::Braced,
        ),
        (
            "\\begin{vmatrix}a&b\\\\c&d\\end{vmatrix}",
            MatrixKind::Barred,
        ),
        (
            "\\begin{Vmatrix}a&b\\\\c&d\\end{Vmatrix}",
            MatrixKind::DoubleBarred,
        ),
        (
            "\\def\\arraystretch{1.5}\\begin{array}{c:c:c}a&b&c\\\\\\hline d&e&f\\\\\\hdashline g&h&i\\end{array}",
            MatrixKind::Plain,
        ),
    ];

    for (latex, expected_kind) in cases {
        let expr = Parser::new(latex)
            .parse()
            .expect("matrix environment parses");
        assert_matrix_kind(&expr, expected_kind);
        let bytes = write_mtef(latex, &expr).expect("matrix environment renders");
        assert!(bytes.len() > 28, "matrix MTEF should include body bytes");
    }
}

/// Ensure related layout environments consume their arguments and render natively.
#[test]
fn layout_environments_parse_and_render_natively() {
    let environments = [
        (
            "\\begin{gather}a=b\\\\c=d\\end{gather}",
            EnvironmentKind::Gather,
        ),
        (
            "\\begin{gathered}a=b\\\\c=d\\end{gathered}",
            EnvironmentKind::Gathered,
        ),
        (
            "\\begin{alignat}{2}10&x+&3&y=2\\\\3&x+&13&y=4\\end{alignat}",
            EnvironmentKind::AlignAt,
        ),
        (
            "\\begin{split}a&=b+c\\\\&=e+f\\end{split}",
            EnvironmentKind::Split,
        ),
        (
            "\\begin{alignedat}{2}10&x+&3&y=2\\\\3&x+&13&y=4\\end{alignedat}",
            EnvironmentKind::AlignedAt,
        ),
        (
            "\\begin{aligned}\\sum_{\\substack{0<i<m\\\\0<j<n}}\\end{aligned}",
            EnvironmentKind::Aligned,
        ),
        (
            "\\begin{rcases}a&b\\\\c&d\\end{rcases}",
            EnvironmentKind::RightCases,
        ),
        (
            "\\begin{dcases}a&b\\\\c&d\\end{dcases}",
            EnvironmentKind::Cases,
        ),
    ];

    for (latex, expected_kind) in environments {
        let expr = Parser::new(latex)
            .parse()
            .expect("layout environment parses");
        assert_environment_kind(&expr, expected_kind);
        assert_no_raw_tex(&expr);
        let bytes = write_mtef(latex, &expr).expect("layout environment renders");
        assert!(
            bytes.len() > 28,
            "layout environment MTEF should include body bytes"
        );
    }

    for latex in [
        "\\begin{array}{cc}a&b\\\\c&d\\end{array}",
        "\\begin{subarray}{l}i\\in\\Lambda\\\\0<j<n\\end{subarray}",
    ] {
        let expr = Parser::new(latex)
            .parse()
            .expect("array environment parses");
        assert_matrix_kind(&expr, MatrixKind::Plain);
        assert_no_raw_tex(&expr);
        let bytes = write_mtef(latex, &expr).expect("array environment renders");
        assert!(bytes.len() > 28, "array MTEF should include body bytes");
    }

    let wrapped = "\\begin{equation}\\begin{split}a&=b+c\\\\&=e+f\\end{split}\\end{equation}";
    let expr = Parser::new(wrapped)
        .parse()
        .expect("equation wrapper parses");
    assert_no_raw_tex(&expr);
    let bytes = write_mtef(wrapped, &expr).expect("equation wrapper renders");
    assert!(
        bytes.len() > 28,
        "equation wrapper MTEF should include body bytes"
    );
}

/// Check the top-level matrix kind without tying the test to row internals.
fn assert_matrix_kind(expr: &Expr, expected: MatrixKind) {
    match expr {
        Expr::Sequence(items) if items.len() == 1 => assert_matrix_kind(&items[0], expected),
        Expr::Style { content, .. } => assert_matrix_kind(content, expected),
        Expr::Matrix { kind, .. } => assert_eq!(*kind, expected),
        other => panic!("expected matrix expression, found {other:?}"),
    }
}

/// Check the top-level environment kind without tying the test to row internals.
fn assert_environment_kind(expr: &Expr, expected: EnvironmentKind) {
    match expr {
        Expr::Sequence(items) if items.len() == 1 => assert_environment_kind(&items[0], expected),
        Expr::Environment { kind, .. } => assert_eq!(*kind, expected),
        other => panic!("expected environment expression, found {other:?}"),
    }
}

/// Ensure \limits and \nolimits attach scripts to the preceding operator.
#[test]
fn limits_modifiers_render_natively() {
    let cases = [
        "\\lim\\limits_{x\\to0} f(x)",
        "\\sum\\limits_{i=1}^{n} x_i",
        "\\prod\\nolimits_{i=1}^{n} x_i",
    ];
    for latex in cases {
        let expr = Parser::new(latex).parse().expect("limits modifier parses");
        assert_no_raw_tex(&expr);
        let bytes = write_mtef(latex, &expr).expect("limits modifier renders");
        assert!(bytes.len() > 28, "limits MTEF should include body bytes");
    }
}

/// Ensure TeX style switches emit MathType logical-size records instead of disappearing.
#[test]
fn style_switches_emit_size_records() {
    let latex = "\\scriptstyle x+\\scriptscriptstyle y+\\displaystyle z+\\textstyle w";
    let expr = Parser::new(latex).parse().expect("style switches parse");
    assert_no_raw_tex(&expr);
    let bytes = write_mtef(latex, &expr).expect("style switches render");
    assert!(
        bytes.contains(&0x0b),
        "scriptstyle should emit MathType's sub-size marker"
    );
    assert!(
        bytes.contains(&0x0c),
        "scriptscriptstyle should emit MathType's sub2-size marker"
    );
}

/// Ensure \char hex escapes are parsed as Unicode while incomplete \char stays raw.
#[test]
fn char_hex_escape_renders_natively() {
    let expr = Parser::new("\\char\"263a")
        .parse()
        .expect("hex char escape parses");
    assert_no_raw_tex(&expr);
    let bytes = write_mtef("\\char\"263a", &expr).expect("hex char escape renders");
    assert!(
        bytes.windows(2).any(|window| window == [0x3a, 0x26]),
        "MTEF should contain U+263A in little-endian form"
    );

    let incomplete = Parser::new("\\char")
        .parse()
        .expect("incomplete char command stays parseable");
    assert!(
        incomplete.contains_raw_tex(),
        "bare \\char should remain raw fallback"
    );
}

/// Ensure \middle accepts a following delimiter while bare \middle stays raw.
#[test]
fn middle_delimiter_renders_natively_when_complete() {
    let latex = "\\left(x\\middle|y\\right)+\\left\\{a\\middle\\vert b\\right\\}+\\vert";
    let expr = Parser::new(latex)
        .parse()
        .expect("complete middle delimiters parse");
    assert_no_raw_tex(&expr);
    let bytes = write_mtef(latex, &expr).expect("middle delimiters render");
    assert!(
        bytes.len() > 28,
        "middle delimiter MTEF should include body bytes"
    );

    let incomplete = Parser::new("\\middle")
        .parse()
        .expect("incomplete middle command stays parseable");
    assert!(
        incomplete.contains_raw_tex(),
        "bare \\middle should remain raw fallback"
    );
}

/// Ensure Supported Functions arrow aliases parse through the generated table.
#[test]
fn supported_arrow_aliases_render_natively() {
    let latex = "\\curvearrowleft+\\curvearrowright+\\dashleftarrow+\\dashrightarrow+\\downdownarrows+\\downharpoonleft+\\downharpoonright+\\hArr+\\hookleftarrow+\\hookrightarrow+\\iff+\\impliedby+\\implies+\\leadsto+\\leftarrowtail+\\leftharpoondown+\\leftleftarrows+\\leftrightarrows+\\leftrightharpoons+\\leftrightsquigarrow+\\longleftarrow+\\longleftrightarrow+\\longmapsto+\\longrightarrow+\\looparrowleft+\\looparrowright+\\nleftarrow+\\nleftrightarrow+\\nrightarrow+\\restriction+\\rightarrowtail+\\rightharpoondown+\\rightleftarrows+\\rightleftharpoons+\\rightrightarrows+\\rightsquigarrow+\\twoheadleftarrow+\\twoheadrightarrow+\\upharpoonleft+\\upharpoonright+\\upuparrows";
    let expr = Parser::new(latex)
        .parse()
        .expect("Supported Functions arrow aliases parse");
    assert_no_raw_tex(&expr);
    let bytes = write_mtef(latex, &expr).expect("Supported Functions arrow aliases render");
    assert!(
        bytes
            .windows(6)
            .any(|window| window == [0x02, 0x04, EXPLICIT_FONT_NEG_1, 0xb6, 0x21, 0xd1]),
        "curvearrowleft should render through the generated command-specific record"
    );
}

/// Ensure Supported Functions x-arrow variants share the native extensible template.
#[test]
fn supported_xarrow_variants_render_natively() {
    let latex = "\\xLeftarrow{abc}+\\xRightarrow{abc}+\\xhookleftarrow{abc}+\\xhookrightarrow{abc}+\\xtwoheadleftarrow{abc}+\\xtwoheadrightarrow{abc}+\\xmapsto{abc}+\\xlongequal{abc}+\\xtofrom{abc}";
    let expr = Parser::new(latex)
        .parse()
        .expect("Supported Functions x-arrow variants parse");
    assert_no_raw_tex(&expr);
    let bytes = write_mtef(latex, &expr).expect("Supported Functions x-arrow variants render");
    assert!(
        bytes
            .windows(5)
            .any(|window| window == [0x02, 0x00, FN_EXPAND, 0xd2, 0x21]),
        "xLeftarrow/xRightarrow should use an expandable double-arrow glyph"
    );
}

/// Ensure single-character \vec uses MathType's EMBELL form for any plain letter.
#[test]
fn single_character_vec_uses_embellishment_for_any_plain_letter() {
    for latex in ["\\vec{x}", "\\vec{a}", "\\vec{z}"] {
        let expr = Parser::new(latex).parse().expect("single-char vec parses");
        assert_no_raw_tex(&expr);
        let bytes = write_mtef(latex, &expr).expect("single-char vec renders");
        assert!(
            bytes.windows(3).any(|window| window == [0x06, 0x00, 0x0b]),
            "{latex} should use MathType's vector EMBELL subtype"
        );
        assert!(
            !bytes
                .windows(5)
                .any(|window| window == [0x03, 0x00, 0x1f, 0x02, 0x00]),
            "{latex} should not fall back to the multi-character tmVEC template"
        );
    }

    let latex = "\\overrightarrow{xy}";
    let expr = Parser::new(latex)
        .parse()
        .expect("multi-char overrightarrow parses");
    let bytes = write_mtef(latex, &expr).expect("multi-char overrightarrow renders");
    assert!(
        bytes
            .windows(5)
            .any(|window| window == [0x03, 0x00, 0x1f, 0x02, 0x00]),
        "multi-character right-arrow accents should still use tmVEC"
    );
}

/// Ensure horizontal bracket accents follow MathType's raw TeX fallback behavior.
#[test]
fn horizontal_brackets_stay_raw_fallback() {
    let latex = "\\overbracket{AB}^{\\text{note}}+\\underbracket{CD}_{\\text{note}}";
    let expr = Parser::new(latex)
        .parse()
        .expect("horizontal brackets parse");
    assert!(
        expr.contains_raw_tex(),
        "horizontal brackets should stay raw"
    );
    let bytes = write_mtef(latex, &expr).expect("horizontal brackets render");
    assert!(
        bytes
            .windows("\\underbracket".len() * 5)
            .any(|window: &[u8]| window.contains(&0x80)),
        "underbracket should write raw TeX CHAR records"
    );
}

/// Ensure MathType-raw x-arrow commands stay raw instead of being over-supported.
#[test]
fn known_raw_xarrow_variants_stay_raw_fallback() {
    let expr = Parser::new("\\xleftrightarrow{abc}")
        .parse()
        .expect("known raw xleftrightarrow parses");
    assert!(
        expr.contains_raw_tex(),
        "MathType stores \\xleftrightarrow as raw text, so it must stay raw"
    );
}

/// Keep standalone aliases on the raw path when MathType TeX Input stores the whole command as text.
#[test]
fn known_raw_simple_aliases_stay_raw_fallback() {
    for latex in [
        "\\arcctg",
        "\\argmax",
        "\\approxcoloncolon",
        "\\cosec",
        "\\dashleftarrow",
        "\\coloneqq",
        "\\copyright",
        "\\hArr",
    ] {
        let expr = Parser::new(latex).parse().expect("raw simple alias parses");
        assert!(
            expr.contains_raw_tex(),
            "MathType stores {latex} as raw text, so it should stay raw"
        );
    }
}

/// Ensure Supported Functions relation aliases parse through the generated table.
#[test]
fn supported_relation_aliases_render_natively() {
    let latex = "\\leqq+\\geqslant+\\lessapprox+\\gtrsim+\\curlyeqprec+\\curlyeqsucc+\\ncong+\\nless+\\nparallel+\\subsetneqq+\\succnapprox+\\trianglelefteq+\\vartriangleright+\\vDash+\\not =+\\not\\in+\\not\\subset+\\not\\Rightarrow+\\dblcolon+\\coloneqq+\\colonequals+\\eqqcolon+\\equalscolon+\\eqcolon+\\minuscolon+\\coloneq+\\colonminus+\\colonapprox+\\colonsim+\\ratio+\\coloncolonequals+\\equalscoloncolon+\\coloncolonminus+\\minuscoloncolon+\\coloncolonapprox+\\coloncolonsim+\\simcoloncolon";
    let expr = Parser::new(latex)
        .parse()
        .expect("Supported Functions relation aliases parse");
    assert_no_raw_tex(&expr);
    let bytes = write_mtef(latex, &expr).expect("Supported Functions relation aliases render");
    assert!(
        bytes
            .windows(5)
            .any(|window| window == [0x02, 0x00, FN_TEXT_FE, 0x66, 0x22]),
        "leqq should render through the generated command-specific table"
    );
    assert!(
        bytes
            .windows(6)
            .any(|window| window == [0x02, 0x04, FN_SYMBOL, 0x60, 0x22, 0xb9]),
        "not-equals should render as a negated relation CHAR record"
    );
}

/// Ensure Supported Functions symbol/text aliases use generated semantic tables.
#[test]
fn supported_symbol_and_text_aliases_render_natively() {
    let latex = "\\bigvee+\\bigwedge+\\daleth+\\gimel+\\diagdown+\\diagup+\\diamonds+\\doublecap+\\doublecup+\\gtrdot+\\image+\\ldotp+\\lgroup x\\rgroup+\\llbracket x\\rrbracket+\\lmoustache x\\rmoustache+\\lozenge+\\maltese+\\mathellipsis+\\measuredangle+\\minuso+\\omicron+\\prime+\\real+\\smallint+\\sphericalangle+\\surd+\\thetasym+\\triangle+\\ulcorner+\\urcorner+\\varDelta+\\varGamma+\\varLambda+\\varOmega+\\varPhi+\\varPi+\\varPsi+\\varSigma+\\varTheta+\\varUpsilon+\\varXi+\\varnothing+\\veebar+\\weierp+\\wr+\\copyright+\\lq+\\mathsterling+\\pounds+\\rq+\\yen";
    let expr = Parser::new(latex)
        .parse()
        .expect("Supported Functions symbol/text aliases parse");
    assert_no_raw_tex(&expr);
    let bytes = write_mtef(latex, &expr).expect("Supported Functions symbol/text aliases render");
    assert!(
        bytes
            .windows(6)
            .any(|window| window == [0x02, 0x04, FN_MT_EXTRA, 0xfd, 0xff, 0x6e]),
        "bigvee should render through the generated MathType big-operator glyph"
    );
    assert!(
        bytes
            .windows(5)
            .any(|window| window == [0x02, 0x00, FN_TEXT, 0xa9, 0x00]),
        "copyright should render as a text CHAR record"
    );
    assert!(
        bytes
            .windows(6)
            .any(|window| window == [0x02, 0x04, FN_LC_GREEK, 0xbf, 0x03, b'o']),
        "omicron should render through the lowercase Greek typeface"
    );
}

/// Ensure common function, font-switch, and text-color aliases stay native.
#[test]
fn semantic_alias_commands_render_natively() {
    let latex = "\\bf Ab0+\\sf Ab0+\\it Ab0+\\textsf{Ab0}+\\textbf{Ab0}+\\bold{Ab0}+\\pmb{\\mu}+\\mathtt{Ab0}+\\texttt{Ab0}+\\verb!x^2!+\\textcolor{blue}{F=ma}+\\text{\\textdegree \\OE \\P \\textcircled a}+\\text{\\'{a} \\`{a} \\~{a} \\={a} \\\"{a} \\H{a} \\.{a} \\v{a} \\^{a} \\u{a} \\r{a}}+\\text{\\sout{abc}}+\\arccos x+\\argmax_x f+\\ch x+\\det A+\\gcd(a,b)+\\inf A+\\cth x+\\th x+\\sinh x+\\tanh x+\\tg x+\\liminf_n x_n+\\operatornamewithlimits{rank}_n A+x\\bmod y+x\\mod y+x\\pmod y+x\\pod y+\\thinspace+\\medspace+\\thickspace+\\negthinspace+\\negmedspace+\\negthickspace+\\space+\\nobreakspace+\\ +\\phase{-78^\\circ}+\\def\\foo{x^2}\\foo+\\gdef\\bar#1{#1^2}\\bar{y}+\\gdef\\VERT{|}+{a \\over b}+{a \\above{2pt} b+1}+\\genfrac ( ] {2pt}{1}a{a+1}+{a \\atop b}+{n \\choose k}+{n \\brace k}+{n \\brack k}+\\sum_{\\substack{0<i<m\\\\0<j<n}}x_{ij}+\\rm Ab0+\\mathrm{Ab0}+\\mathnormal{Ab0}+\\textnormal{Ab0}+\\textup{Ab0}+\\textmd{Ab0}+\\mathit{Ab0}+\\textit{Ab0}+\\emph{Ab0}+\\bm{Ab0}+\\boldsymbol{xy}+\\lt+\\gt+\\colon+\\clubs+\\hearts+\\spades+\\degree+\\left\\lt x \\right\\gt+\\langle x\\rangle+\\lbrace y\\rbrace+\\lbrack z\\rbrack+\\lVert v\\rVert+\\intop f+\\iiint f+\\oiint f+\\oiiint f+\\overleftarrow{AB}+\\overrightarrow{AB}+\\underleftarrow{AB}+\\underrightarrow{AB}+\\overleftrightarrow{AB}+\\underleftrightarrow{AB}+\\overleftharpoon{ac}+\\overrightharpoon{ac}+\\xleftarrow{abc}+\\xrightarrow[under]{over}+\\overline{AB}+\\underline{CD}+\\underbar{X}+\\u{a}+\\v{a}+\\widecheck{ac}+\\cancel{5}+\\bcancel{5}+\\xcancel{ABC}+\\sout{abc}+\\stackrel{!}{=}+\\overset{!}{=}+\\underset{!}{=}+\\cal AB0";
    let expr = Parser::new(latex)
        .parse()
        .expect("semantic alias commands parse");
    assert_no_raw_tex(&expr);
    let bytes = write_mtef(latex, &expr).expect("semantic alias commands render");
    assert!(
        bytes.len() > 28,
        "semantic alias MTEF should include body bytes"
    );
    assert!(
        bytes
            .windows(5)
            .any(|window| window == [0x02, 0x00, FN_USER1, b'A', 0x00]),
        "mathtt/texttt should render through MathType's Courier New user style"
    );
}

/// Keep MathType's known hybrid wrappers parseable even when they stay raw-text-prefixed.
#[test]
fn semantic_alias_hybrid_wrappers_preserve_raw_prefixes() {
    let cases = [
        "\\bra{\\phi}",
        "\\ket{\\psi}",
        "\\braket{\\phi\\VERT\\psi}",
        "\\Braket{\\phi\\VERT\\psi}",
        "\\Overrightarrow{AB}",
        "\\Set{x\\VERT x<5}",
        "\\boxed{\\pi=\\frac c d}",
        "\\sum_{\\mathclap{1\\le i\\le n}}x_i",
        "{=}\\mathllap{/\\,}",
        "\\mathrlap{\\,/}{=}",
    ];
    for latex in cases {
        let expr = Parser::new(latex)
            .parse()
            .expect("hybrid MathType wrapper parses");
        assert!(
            expr.contains_raw_tex(),
            "hybrid MathType wrapper should preserve raw command text: {latex}"
        );
        let bytes = write_mtef(latex, &expr).expect("hybrid MathType wrapper renders");
        assert!(
            bytes.len() > 28,
            "hybrid MathType wrapper should include body bytes: {latex}"
        );
    }
}

/// Ensure MathType-native spacing aliases use the probed fnSPACE records.
#[test]
fn spacing_aliases_render_natively() {
    let latex = "\\medspace x+\\thickspace y+\\negmedspace z+\\negthickspace w";
    let expr = Parser::new(latex).parse().expect("spacing aliases parse");
    assert_no_raw_tex(&expr);
    let bytes = write_mtef(latex, &expr).expect("spacing aliases render");
    for width in [0x02, 0x04, 0x01] {
        assert!(
            bytes
                .windows(5)
                .any(|window| window == [0x02, 0x00, FN_SPACE, width, 0xef]),
            "spacing alias should render fnSPACE width 0x{width:02x}"
        );
    }
}

/// Ensure escaped punctuation and one-character spacing aliases follow MathType semantics.
#[test]
fn escaped_punctuation_and_spacing_render_natively() {
    let latex = "\\#+\\%+\\&+\\:+\\;+\\>+\\ ";
    let expr = Parser::new(latex)
        .parse()
        .expect("escaped punctuation and spacing parse");
    assert_no_raw_tex(&expr);
    let bytes = write_mtef(latex, &expr).expect("escaped punctuation and spacing render");
    for ch in ['#', '%', '&'] {
        assert!(
            bytes
                .windows(5)
                .any(|window| window == [0x02, 0x00, FN_FUNCTION, ch as u8, 0x00]),
            "escaped punctuation should render as function-style CHAR {ch}"
        );
    }
    for width in [0x02, 0x04, 0x08] {
        assert!(
            bytes
                .windows(5)
                .any(|window| window == [0x02, 0x00, FN_SPACE, width, 0xef]),
            "escaped spacing should render fnSPACE width 0x{width:02x}"
        );
    }
}

/// Ensure simple \utilde groups use MathType's documented under-tilde embellishment.
#[test]
fn simple_under_tilde_renders_natively() {
    let latex = "\\utilde{AB}";
    let expr = Parser::new(latex).parse().expect("simple utilde parses");
    assert_no_raw_tex(&expr);
    let bytes = write_mtef(latex, &expr).expect("simple utilde renders");
    let under_tilde_count = bytes
        .windows(3)
        .filter(|window| *window == [0x06, 0x00, 0x1e])
        .count();
    assert_eq!(
        under_tilde_count, 2,
        "each simple utilde character should carry embU_TILDE"
    );
}

/// Ensure metadata/layout wrappers keep their visible math content native.
#[test]
fn content_wrapper_commands_render_natively() {
    let latex = "\\htmlId{bar}{x}+\\htmlClass{foo}{y}+\\htmlStyle{color:red;}{z}+\\htmlData{foo=a}{w}+\\colorbox{aqua}{$F=ma$}+\\fcolorbox{red}{aqua}{$E=mc^2$}+a\\raisebox{0.25em}{$b$}c+\\textrm{Ab0}+\\tt Ab0+\\sqrt{\\smash[b]{y}}+\\left(\\vcenter{\\frac{\\frac a b}c}\\right)";
    let expr = Parser::new(latex)
        .parse()
        .expect("content wrapper commands parse");
    assert_no_raw_tex(&expr);
    let bytes = write_mtef(latex, &expr).expect("content wrapper commands render");
    assert!(
        bytes
            .windows(5)
            .any(|window| window == [0x02, 0x00, FN_USER1, b'A', 0x00]),
        "tt should render through MathType's Courier New user style"
    );
}

/// Ensure incomplete pmb stays raw while braced pmb uses the bold-font path.
#[test]
fn pmb_requires_an_argument_for_native_rendering() {
    let bare = Parser::new("\\pmb").parse().expect("bare pmb parses");
    assert!(
        bare.contains_raw_tex(),
        "bare \\pmb is incomplete and should remain raw"
    );

    let expr = Parser::new("\\pmb{\\mu}")
        .parse()
        .expect("braced pmb parses");
    assert_no_raw_tex(&expr);
    let bytes = write_mtef("\\pmb{\\mu}", &expr).expect("braced pmb renders");
    assert!(
        bytes.len() > 28,
        "braced pmb should render through the native bold-font path"
    );
}

/// Ensure standard blackboard aliases reuse the generated mathbb table.
#[test]
fn blackboard_aliases_render_natively() {
    let latex = "\\Bbb{AB}+\\Complex+\\cnums+\\Reals+\\reals+\\N+\\natnums+\\Z";
    let expr = Parser::new(latex)
        .parse()
        .expect("blackboard aliases parse");
    assert_no_raw_tex(&expr);
    let bytes = write_mtef(latex, &expr).expect("blackboard aliases render");
    assert!(bytes.len() > 28, "alias MTEF should include body bytes");
}

/// Ensure \mathfrak uses the generated table, including MathType's special I record.
#[test]
fn mathfrak_renders_from_generated_table() {
    let latex = "\\mathfrak{AIz}";
    let expr = Parser::new(latex).parse().expect("mathfrak parses");
    assert_no_raw_tex(&expr);
    let bytes = write_mtef(latex, &expr).expect("mathfrak renders");
    assert!(
        bytes
            .windows(6)
            .any(|window| window == [0x02, 0x04, FN_SYMBOL, 0x11, 0x21, 0xc1]),
        "mathfrak I should use the generated MathType special CHAR record"
    );
}

/// Ensure text mode can emit UTF-16 surrogate pairs for non-BMP symbols.
#[test]
fn non_bmp_text_renders_as_utf16_char_records() {
    let latex = "\\text{𝐀-𝟗}";
    let expr = Parser::new(latex).parse().expect("non-BMP text parses");
    assert_no_raw_tex(&expr);
    let bytes = write_mtef(latex, &expr).expect("non-BMP text renders");
    assert!(
        bytes.windows(2).any(|window| window == [0x35, 0xd8]),
        "MTEF should contain a high-surrogate code unit"
    );
}

/// Preserve unsupported environments as raw TeX instead of failing parsing.
#[test]
fn unsupported_environment_renders_as_raw_fallback() {
    let latex = "\\begin{CD}A @>a>> B \\\\ C @= D\\end{CD}";
    let expr = Parser::new(latex)
        .parse()
        .expect("unsupported environment parses as fallback");
    assert!(
        expr.contains_raw_tex(),
        "unsupported environment should stay visibly raw"
    );
    let bytes = write_mtef(latex, &expr).expect("unsupported environment renders");
    assert!(
        bytes.len() > 28,
        "raw fallback MTEF should include body bytes"
    );
}

/// Reject raw TeX fallback in tests that claim native parser support.
fn assert_no_raw_tex(expr: &Expr) {
    match expr {
        Expr::RawTex(text) => panic!("unexpected raw TeX fallback: {text}"),
        Expr::Sequence(items) => items.iter().for_each(assert_no_raw_tex),
        Expr::Color { content, .. }
        | Expr::Style { content, .. }
        | Expr::Font { content, .. }
        | Expr::Accent { content, .. }
        | Expr::ArrowAccent { content, .. }
        | Expr::BarTemplate { content, .. }
        | Expr::Strike { content, .. }
        | Expr::Boxed(content)
        | Expr::Sqrt(content)
        | Expr::Delimited { content, .. }
        | Expr::Script { base: content, .. } => assert_no_raw_tex(content),
        Expr::Fraction(left, right)
        | Expr::Stackrel {
            upper: left,
            lower: right,
        }
        | Expr::Underset {
            lower: left,
            base: right,
        } => {
            assert_no_raw_tex(left);
            assert_no_raw_tex(right);
        }
        Expr::Pile { upper, lower, .. } => {
            assert_no_raw_tex(upper);
            assert_no_raw_tex(lower);
        }
        Expr::NthRoot { index, radicand } => {
            assert_no_raw_tex(index);
            assert_no_raw_tex(radicand);
        }
        Expr::BigOp {
            lower, upper, body, ..
        }
        | Expr::IntegralOp {
            lower, upper, body, ..
        } => {
            lower.as_deref().into_iter().for_each(assert_no_raw_tex);
            upper.as_deref().into_iter().for_each(assert_no_raw_tex);
            body.as_deref().into_iter().for_each(assert_no_raw_tex);
        }
        Expr::Limit { lower, upper, .. } => {
            lower.as_deref().into_iter().for_each(assert_no_raw_tex);
            upper.as_deref().into_iter().for_each(assert_no_raw_tex);
        }
        Expr::Brace {
            content,
            annotation,
            ..
        } => {
            assert_no_raw_tex(content);
            annotation
                .as_deref()
                .into_iter()
                .for_each(assert_no_raw_tex);
        }
        Expr::XArrow { label, under, .. } => {
            assert_no_raw_tex(label);
            under.as_deref().into_iter().for_each(assert_no_raw_tex);
        }
        Expr::Matrix { rows, .. } | Expr::Environment { rows, .. } => rows
            .iter()
            .flat_map(|row| row.iter())
            .for_each(assert_no_raw_tex),
        Expr::Char(_)
        | Expr::CommandSymbol { .. }
        | Expr::BigSymbol(_)
        | Expr::SumOperatorSymbol(_)
        | Expr::Space(_)
        | Expr::FunctionName(_)
        | Expr::Text(_)
        | Expr::Integral { .. } => {}
    }
}

/// Render MTEF through the same parser/writer path as the CLI.
fn render_mtef_for_test(latex: &str) -> Result<Vec<u8>, String> {
    let expr = Parser::new(latex).parse()?;
    write_mtef(latex, &expr)
}
