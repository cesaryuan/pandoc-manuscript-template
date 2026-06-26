#![allow(dead_code)]

/// Rewrite LaTeX constructs that MathType's TeX Input translator rejects.
pub(crate) fn normalize_mathtype_latex(latex: &str) -> String {
    rewrite_environment_name(latex, "aligned", "align")
}

/// Wrap TeX in the math-delimited payload form accepted by MathType OLE.
pub(crate) fn mathtype_tex_payload(latex: &str) -> String {
    let text = normalize_mathtype_latex(latex.trim());
    if text.starts_with("$$") && text.ends_with("$$") {
        return text;
    }
    if text.starts_with('$') && text.ends_with('$') {
        return text;
    }
    if text.starts_with(r"\(") && text.ends_with(r"\)") {
        return format!("${}$", text[2..text.len() - 2].trim());
    }
    if text.starts_with(r"\[") && text.ends_with(r"\]") {
        return format!("$${}$$", text[2..text.len() - 2].trim());
    }
    format!("${text}$")
}

/// Rewrite one environment name through \begin/\end while tolerating extra spaces.
fn rewrite_environment_name(input: &str, from: &str, to: &str) -> String {
    let chars = input.chars().collect::<Vec<_>>();
    let mut output = String::with_capacity(input.len());
    let mut index = 0usize;
    while index < chars.len() {
        if chars[index] != '\\' {
            output.push(chars[index]);
            index += 1;
            continue;
        }
        if let Some((replacement, consumed)) = rewrite_environment_at(&chars[index..], from, to) {
            output.push_str(&replacement);
            index += consumed;
            continue;
        }
        output.push(chars[index]);
        index += 1;
    }
    output
}

/// Return a canonicalized rewritten environment command starting at one backslash.
fn rewrite_environment_at(chars: &[char], from: &str, to: &str) -> Option<(String, usize)> {
    for command in ["begin", "end"] {
        if !matches_command(chars, command) {
            continue;
        }
        let mut index = 1 + command.len();
        while chars.get(index).is_some_and(|ch| ch.is_whitespace()) {
            index += 1;
        }
        if chars.get(index) != Some(&'{') {
            return None;
        }
        index += 1;
        while chars.get(index).is_some_and(|ch| ch.is_whitespace()) {
            index += 1;
        }
        let env_start = index;
        while chars.get(index).is_some_and(|ch| ch.is_ascii_alphabetic()) {
            index += 1;
        }
        if env_start == index {
            return None;
        }
        let env_name = chars[env_start..index].iter().collect::<String>();
        while chars.get(index).is_some_and(|ch| ch.is_whitespace()) {
            index += 1;
        }
        if chars.get(index) != Some(&'}') {
            return None;
        }
        index += 1;
        if env_name != from {
            return None;
        }
        return Some((format!(r"\{command}{{{to}}}"), index));
    }
    None
}

/// Return true when the slice begins with one control word command name.
fn matches_command(chars: &[char], command: &str) -> bool {
    if chars.first() != Some(&'\\') {
        return false;
    }
    let command_chars = command.chars().collect::<Vec<_>>();
    chars.get(1..1 + command_chars.len()) == Some(command_chars.as_slice())
}

#[cfg(test)]
mod tests {
    use super::{mathtype_tex_payload, normalize_mathtype_latex};

    /// MathType rejects bare fragments in OLE SetData, so helper probes must add delimiters.
    #[test]
    fn wraps_bare_math_for_mathtype_payload() {
        assert_eq!(mathtype_tex_payload("x+1"), "$x+1$");
        assert_eq!(mathtype_tex_payload(r"\(x+1\)"), "$x+1$");
        assert_eq!(mathtype_tex_payload(r"\[x+1\]"), "$$x+1$$");
        assert_eq!(mathtype_tex_payload("$$x+1$$"), "$$x+1$$");
    }

    /// MathType TeX Input rejects aligned but accepts align for the same row content.
    #[test]
    fn rewrites_aligned_environment_for_mathtype() {
        assert_eq!(
            normalize_mathtype_latex(r"\begin{aligned}a&=b\\c&=d\end{aligned}"),
            r"\begin{align}a&=b\\c&=d\end{align}"
        );
        assert_eq!(
            normalize_mathtype_latex(r"\begin { aligned }x\end { aligned }"),
            r"\begin{align}x\end{align}"
        );
    }
}
