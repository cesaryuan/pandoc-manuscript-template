#![allow(dead_code)]

/// Wrap TeX in the math-delimited payload form accepted by MathType OLE.
pub(crate) fn mathtype_tex_payload(latex: &str) -> String {
    let text = latex.trim();
    if text.starts_with("$$") && text.ends_with("$$") {
        return text.to_string();
    }
    if text.starts_with('$') && text.ends_with('$') {
        return text.to_string();
    }
    if text.starts_with(r"\(") && text.ends_with(r"\)") {
        return format!("${}$", text[2..text.len() - 2].trim());
    }
    if text.starts_with(r"\[") && text.ends_with(r"\]") {
        return format!("$${}$$", text[2..text.len() - 2].trim());
    }
    format!("${text}$")
}

#[cfg(test)]
mod tests {
    use super::mathtype_tex_payload;

    /// MathType rejects bare fragments in OLE SetData, so helper probes must add delimiters.
    #[test]
    fn wraps_bare_math_for_mathtype_payload() {
        assert_eq!(mathtype_tex_payload("x+1"), "$x+1$");
        assert_eq!(mathtype_tex_payload(r"\(x+1\)"), "$x+1$");
        assert_eq!(mathtype_tex_payload(r"\[x+1\]"), "$$x+1$$");
        assert_eq!(mathtype_tex_payload("$$x+1$$"), "$$x+1$$");
    }

    /// Keep helper probes faithful to the source so unsupported environments
    /// such as `aligned` can still be audited against their real fallback bytes.
    #[test]
    fn preserves_aligned_environment_in_payload() {
        assert_eq!(
            mathtype_tex_payload(r"\begin{aligned}a&=b\\c&=d\end{aligned}"),
            r"$\begin{aligned}a&=b\\c&=d\end{aligned}$"
        );
    }
}
