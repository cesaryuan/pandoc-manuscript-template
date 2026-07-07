use super::*;

impl Parser {
    /// Parse one `\newcommand`-style definition as raw prefix plus visible arguments.
    ///
    /// Bug-fix: MathType keeps these definitions visible at the definition site
    /// but does not make later invocations expand to the replacement text. A
    /// second optional bracket after the arity, such as `[1][+]`, causes the
    /// whole formula to collapse into `(Tex translation failed)`.
    pub(super) fn parse_command_definition(&mut self, command: &str) -> Result<Expr, String> {
        let checkpoint = self.pos;
        let name = match self.parse_command_definition_name(command) {
            Ok(name) => name,
            Err(_) => {
                self.pos = checkpoint;
                return Ok(Expr::RawTex(format!("\\{command}")));
            }
        };
        let mut args = vec![name];
        if let Some(arity) = self.parse_optional_bracket_group()? {
            args.push(Expr::Sequence(vec![
                Expr::Char('['),
                arity,
                Expr::Char(']'),
            ]));
            self.skip_ws();
            if self.peek() == Some('[') {
                self.pos = self.chars.len();
                return Ok(Expr::Text(MATHTYPE_TEXT_TRANSLATION_FAILED.to_string()));
            }
        }
        let replacement = match self.parse_visible_wrapper_group(&format!("{command} replacement"))
        {
            Ok(replacement) => normalize_definition_fallback_expr(replacement),
            Err(_) => {
                self.pos = checkpoint;
                return Ok(Expr::RawTex(format!("\\{command}")));
            }
        };
        args.push(replacement);
        Ok(raw_prefix_sequence(command, args))
    }

    /// Parse one `\newenvironment`-style definition as raw prefix plus visible arguments.
    ///
    /// Bug-fix: MathType keeps these definitions visible at the definition site
    /// but does not register the environment for later `\begin...\end` expansion.
    /// Replacement groups that contain unmatched `\begin` / `\end` stay partially
    /// raw with visible braces, while a fully supported balanced environment body
    /// collapses the whole formula into `(Tex translation failed)`.
    pub(super) fn parse_environment_definition(&mut self, command: &str) -> Result<Expr, String> {
        let checkpoint = self.pos;
        let name = match self.parse_visible_wrapper_group(&format!("{command} name")) {
            Ok(name) => name,
            Err(_) => {
                self.pos = checkpoint;
                return Ok(Expr::RawTex(format!("\\{command}")));
            }
        };
        let mut args = vec![name];
        if let Some(arity) = self.parse_optional_bracket_group()? {
            args.push(Expr::Sequence(vec![
                Expr::Char('['),
                arity,
                Expr::Char(']'),
            ]));
        }
        if let Some(default) = self.parse_optional_bracket_group()? {
            args.push(Expr::Sequence(vec![
                Expr::Char('['),
                default,
                Expr::Char(']'),
            ]));
        }
        let begin = match self.parse_environment_definition_body(&format!("{command} begin")) {
            Ok(begin) => begin,
            Err(_) => {
                self.pos = checkpoint;
                return Ok(Expr::RawTex(format!("\\{command}")));
            }
        };
        if expr_is_mathtype_translation_failed(&begin) {
            self.pos = self.chars.len();
            return Ok(begin);
        }
        args.push(begin);
        let end = match self.parse_environment_definition_body(&format!("{command} end")) {
            Ok(end) => end,
            Err(_) => {
                self.pos = checkpoint;
                return Ok(Expr::RawTex(format!("\\{command}")));
            }
        };
        if expr_is_mathtype_translation_failed(&end) {
            self.pos = self.chars.len();
            return Ok(end);
        }
        args.push(end);
        Ok(raw_prefix_sequence(command, args))
    }

    /// Parse a narrow \def/\gdef subset used by Supported Functions examples.
    pub(super) fn parse_macro_definition(&mut self, command: &str) -> Result<Expr, String> {
        let checkpoint = self.pos;
        self.skip_ws();
        let Some(name) = self.parse_macro_name()? else {
            if self.peek() == Some('{') {
                self.pos = checkpoint;
                return self.parse_malformed_braced_macro_definition(command);
            }
            return Ok(Expr::RawTex(format!("\\{command}")));
        };
        let params = self.parse_macro_parameter_count()?;
        if self.peek() == Some('[') {
            // Bug-fix: MathType rejects \def signatures that start an optional-style
            // bracketed parameter pattern after the control sequence name.
            self.pos = self.chars.len();
            return Ok(Expr::Text(MATHTYPE_TEXT_TRANSLATION_FAILED.to_string()));
        }
        let replacement = match self.parse_raw_group("macro replacement") {
            Ok(replacement) => replacement,
            Err(_) => {
                // Bug-fix: malformed macro definitions should preserve the
                // defining control word raw instead of failing the whole sample.
                self.pos = checkpoint;
                return Ok(Expr::RawTex(format!("\\{command}")));
            }
        };
        let rendered = self.render_macro_definition(command, &name, params, &replacement)?;
        self.macros.insert(
            name.clone(),
            MacroDefinition {
                params,
                replacement: replacement.clone(),
                render_mode: MacroRenderMode::RawOnly,
            },
        );
        Ok(rendered)
    }

    /// Preserve malformed `\def{...}` names as one raw prefix plus visible signature fragments.
    ///
    /// Bug-fix: MathType does not reject `\def{\bar}[#1]#2{}` outright. It keeps
    /// the braced name raw, exposes the following parameter markers visibly, and
    /// ignores an empty trailing replacement group.
    fn parse_malformed_braced_macro_definition(&mut self, command: &str) -> Result<Expr, String> {
        self.skip_ws();
        let raw_name = self.parse_raw_group("malformed macro name")?;
        let mut items = vec![Expr::RawTex(format!("\\{command}{{{raw_name}}}"))];
        while let Some(group) = self.parse_optional_bracket_group()? {
            items.push(Expr::Char('['));
            push_visible_items(&mut items, group);
            items.push(Expr::Char(']'));
        }
        loop {
            let checkpoint = self.pos;
            self.skip_ws();
            if self.peek() != Some('#') {
                self.pos = checkpoint;
                break;
            }
            self.pos += 1;
            let Some(digit @ ('1'..='9')) = self.peek() else {
                self.pos = checkpoint;
                break;
            };
            self.pos += 1;
            items.push(Expr::RawTex("#".to_string()));
            items.push(Expr::Char(digit));
        }
        let checkpoint = self.pos;
        if let Ok(replacement) = self.parse_raw_group("malformed macro replacement") {
            if !replacement.is_empty() {
                push_visible_items(
                    &mut items,
                    normalize_definition_fallback_expr(
                        self.parse_visible_wrapper_text(&replacement)?,
                    ),
                );
            }
        } else {
            self.pos = checkpoint;
        }
        Ok(collapse_single_sequence(Expr::Sequence(items)))
    }

    /// Expand a previously defined simple macro command, if one is in scope.
    pub(super) fn expand_macro_command(&mut self, command: &str) -> Result<Option<Expr>, String> {
        let Some(definition) = self.macros.get(command).cloned() else {
            return Ok(None);
        };
        match definition.render_mode {
            MacroRenderMode::RawOnly => {
                self.render_raw_macro_invocation(command, definition.params)
            }
        }
    }

    /// Parse a replacement string with the current macro scope and depth limit.
    pub(super) fn parse_macro_replacement(&self, replacement: &str) -> Result<Expr, String> {
        let mut parser = Parser {
            chars: replacement.chars().collect(),
            pos: 0,
            macros: self.macros.clone(),
            let_aliases: self.let_aliases.clone(),
            expansion_depth: self.expansion_depth + 1,
            pending_raw_ws: String::new(),
            active_unsupported_envs: self.active_unsupported_envs.clone(),
            pending_closed_unsupported_envs: self.pending_closed_unsupported_envs.clone(),
        };
        let expr = parser.parse_sequence(None)?;
        parser.skip_ws();
        if parser.pos != parser.chars.len() {
            return Err(format!(
                "unexpected character in macro replacement {:?}",
                parser.peek()
            ));
        }
        Ok(expr)
    }

    /// Render one raw-only macro definition the way MathType TeX Input exposes it.
    pub(super) fn render_macro_definition(
        &self,
        command: &str,
        name: &str,
        params: usize,
        replacement: &str,
    ) -> Result<Expr, String> {
        if params == 0 {
            if let Some(applied) = self.render_macro_name_application(name, replacement)? {
                return Ok(raw_prefix_sequence_with_raw(
                    format!("\\{command}"),
                    vec![applied],
                ));
            }
        }
        let mut args = Vec::new();
        let raw_prefix = match params {
            0 => format!("\\{command}\\{name}"),
            1 => {
                args.push(Expr::Char('1'));
                format!("\\{command}\\{name}#")
            }
            2 => {
                args.push(Expr::Char('1'));
                args.push(Expr::RawTex("#".to_string()));
                args.push(Expr::Char('2'));
                format!("\\{command}\\{name}#")
            }
            _ => return Ok(Expr::RawTex(format!("\\{command}"))),
        };
        let rendered_replacement = render_raw_only_macro_replacement(replacement, self)?;
        if params == 0 && replacement_is_pure_raw_tex(&rendered_replacement) {
            return Ok(Expr::Sequence(vec![
                Expr::RawTex(raw_prefix),
                Expr::RawBoundary,
                Expr::RawTex(format!("{{{replacement}}}")),
            ]));
        }
        args.extend(rendered_replacement);
        Ok(raw_prefix_sequence_with_raw(raw_prefix, args))
    }

    /// Render a macro name through its native one-argument meaning when MathType does so.
    pub(super) fn render_macro_name_application(
        &self,
        name: &str,
        replacement: &str,
    ) -> Result<Option<Expr>, String> {
        let mut macros = self.macros.clone();
        macros.remove(name);
        let mut parser = Parser {
            chars: format!("\\{name}{{{replacement}}}").chars().collect(),
            pos: 0,
            macros,
            let_aliases: self.let_aliases.clone(),
            expansion_depth: self.expansion_depth + 1,
            pending_raw_ws: String::new(),
            active_unsupported_envs: self.active_unsupported_envs.clone(),
            pending_closed_unsupported_envs: self.pending_closed_unsupported_envs.clone(),
        };
        let expr = parser.parse_sequence(None)?;
        parser.skip_ws();
        if parser.pos != parser.chars.len() {
            return Ok(None);
        }
        Ok((!expr.contains_raw_tex()).then_some(expr))
    }
    /// Render one raw-only macro invocation without expanding it into native MathType structures.
    pub(super) fn render_raw_macro_invocation(
        &mut self,
        command: &str,
        params: usize,
    ) -> Result<Option<Expr>, String> {
        match params {
            0 => Ok(Some(Expr::RawTex(format!("\\{command}")))),
            1 => {
                let argument = match self.parse_raw_group("macro argument") {
                    Ok(argument) => argument,
                    Err(_) => return Ok(Some(Expr::RawTex(format!("\\{command}")))),
                };
                Ok(Some(raw_prefix_expr(
                    command,
                    self.parse_visible_wrapper_text(&argument)?,
                )))
            }
            2 => {
                let first = match self.parse_raw_group("macro first argument") {
                    Ok(argument) => self.parse_visible_wrapper_text(&argument)?,
                    Err(_) => return Ok(Some(Expr::RawTex(format!("\\{command}")))),
                };
                let second = match self.parse_raw_group("macro second argument") {
                    Ok(argument) => self.parse_visible_wrapper_text(&argument)?,
                    Err(_) => return Ok(Some(raw_prefix_expr(command, first))),
                };
                Ok(Some(raw_prefix_sequence(command, vec![first, second])))
            }
            _ => Ok(Some(Expr::RawTex(format!("\\{command}")))),
        }
    }

    /// Parse the control-word name after \def or \gdef.
    pub(super) fn parse_macro_name(&mut self) -> Result<Option<String>, String> {
        self.skip_ws();
        if self.peek() != Some('\\') {
            return Ok(None);
        }
        self.pos += 1;
        let start = self.pos;
        while self.peek().is_some_and(|ch| ch.is_ascii_alphabetic()) {
            self.pos += 1;
        }
        if self.pos == start {
            return Err("expected macro control-word name".to_string());
        }
        Ok(Some(self.chars[start..self.pos].iter().collect()))
    }

    /// Parse the supported macro signatures: no parameters, `#1`, or `#1#2`.
    pub(super) fn parse_macro_parameter_count(&mut self) -> Result<usize, String> {
        self.skip_ws();
        if self.peek() != Some('#') {
            return Ok(0);
        }
        let mut params = 0usize;
        while self.peek() == Some('#') {
            self.pos += 1;
            params += 1;
            let expected = char::from_digit(params as u32, 10)
                .ok_or_else(|| "only up to two macro parameters are supported".to_string())?;
            if self.peek() != Some(expected) {
                return Err(format!(
                    "expected macro parameter marker #{expected} after #"
                ));
            }
            self.pos += 1;
            if params == 2 {
                break;
            }
        }
        if self.peek() == Some('#') {
            return Err("only up to two macro parameters are supported".to_string());
        }
        Ok(params)
    }

    /// Keep HTML wrapper attributes visible because MathType emits them after the raw command name.
    pub(super) fn parse_html_wrapper_content(&mut self, command: &str) -> Result<Expr, String> {
        let attribute = self.parse_visible_wrapper_group(&format!("{command} attribute"))?;
        let content = self.parse_visible_wrapper_group(&format!("{command} content"))?;
        Ok(raw_prefix_sequence(command, vec![attribute, content]))
    }

    /// Keep color-box arguments visible because MathType preserves them after the raw command name.
    pub(super) fn parse_color_box_content(&mut self, has_frame: bool) -> Result<Expr, String> {
        let mut args = Vec::with_capacity(3);
        if has_frame {
            args.push(self.parse_visible_wrapper_group("fcolorbox frame color")?);
        }
        args.push(self.parse_visible_wrapper_group("colorbox background color")?);
        args.push(self.parse_visible_wrapper_group("colorbox content")?);
        Ok(raw_prefix_sequence(
            if has_frame { "fcolorbox" } else { "colorbox" },
            args,
        ))
    }

    /// Preserve MathType's hybrid \genfrac path: raw command name plus visible arguments.
    pub(super) fn parse_genfrac(&mut self) -> Result<Expr, String> {
        let left = self.parse_genfrac_delimiter("genfrac left delimiter")?;
        let right = self.parse_genfrac_delimiter("genfrac right delimiter")?;
        let thickness = self.parse_raw_group("genfrac line thickness")?;
        let style = self.parse_raw_group("genfrac style")?;
        let numerator = self.parse_required_group_or_atom("genfrac numerator")?;
        let denominator = self.parse_required_group_or_atom("genfrac denominator")?;
        let mut args = Vec::with_capacity(6);
        if let Some(left) = left {
            args.push(Expr::Char(left));
        }
        if let Some(right) = right {
            args.push(Expr::Char(right));
        }
        if !thickness.is_empty() {
            args.push(self.parse_visible_wrapper_text(&thickness)?);
        }
        if !style.is_empty() {
            args.push(self.parse_visible_wrapper_text(&style)?);
        }
        args.push(numerator);
        args.push(denominator);
        Ok(raw_prefix_sequence("genfrac", args))
    }

    /// Parse a \genfrac delimiter, where `{}` means no delimiter.
    pub(super) fn parse_genfrac_delimiter(&mut self, label: &str) -> Result<Option<char>, String> {
        self.skip_ws();
        if self.peek() == Some('{') {
            let raw = self.parse_raw_group(label)?;
            return if raw.trim().is_empty() {
                Ok(None)
            } else {
                let mut parser = Parser::new(&raw);
                Ok(Some(parser.parse_delimiter_char(label)?))
            };
        }
        self.parse_delimiter_char(label).map(Some)
    }

    /// Parse one raw wrapper group as visible content so MathType-style hybrid wrappers can reuse it.
    pub(super) fn parse_visible_wrapper_group(&mut self, label: &str) -> Result<Expr, String> {
        let content = self.parse_raw_group(label)?;
        self.parse_visible_wrapper_text(&content)
    }

    /// Parse the defined control-sequence name kept visible after `\newcommand`-style prefixes.
    pub(super) fn parse_command_definition_name(&mut self, command: &str) -> Result<Expr, String> {
        let content = self.parse_raw_group(&format!("{command} name"))?;
        if let Some(rest) = content.strip_prefix('\\') {
            if !rest
                .chars()
                .next()
                .is_some_and(|ch| ch.is_ascii_alphabetic())
            {
                let mut items = vec![Expr::RawTex("\\".to_string())];
                push_visible_items(&mut items, self.parse_visible_wrapper_text(rest)?);
                return Ok(Expr::Sequence(items));
            }
        }
        Ok(normalize_definition_fallback_expr(
            self.parse_visible_wrapper_text(&content)?,
        ))
    }

    /// Parse one `\newenvironment` replacement group with MathType's mixed raw/visible rules.
    pub(super) fn parse_environment_definition_body(
        &mut self,
        label: &str,
    ) -> Result<Expr, String> {
        let content = self.parse_raw_group(label)?;
        let visible =
            normalize_definition_fallback_expr(self.parse_visible_wrapper_text(&content)?);
        if !content.contains("\\begin{") && !content.contains("\\end{") {
            return Ok(visible);
        }
        if is_balanced_supported_environment_body(&visible) {
            return Ok(Expr::Text(MATHTYPE_TEXT_TRANSLATION_FAILED.to_string()));
        }
        Ok(self.wrap_environment_definition_body_fragments(&content)?)
    }

    /// Parse one raw wrapper fragment so unsupported wrapper arguments stay visible in MathType order.
    pub(super) fn parse_visible_wrapper_text(&self, content: &str) -> Result<Expr, String> {
        self.parse_macro_replacement(content)
    }

    /// Keep `\begin{...}` / `\end{...}` wrappers raw inside environment definitions
    /// while still parsing the body content between them visibly.
    fn wrap_environment_definition_body_fragments(&self, content: &str) -> Result<Expr, String> {
        let mut items = vec![Expr::RawTex("{".to_string())];
        for fragment in split_environment_definition_body_fragments(content) {
            match fragment {
                EnvironmentDefinitionFragment::Raw(text) => items.push(Expr::RawTex(text)),
                EnvironmentDefinitionFragment::Visible(text) => {
                    if text.is_empty() {
                        continue;
                    }
                    push_visible_items(
                        &mut items,
                        normalize_definition_fallback_expr(self.parse_visible_wrapper_text(&text)?),
                    );
                }
                EnvironmentDefinitionFragment::MatchedWrapper {
                    leading_ws,
                    command,
                    name,
                } => {
                    if !leading_ws.is_empty() {
                        items.push(Expr::RawTex(leading_ws));
                    }
                    items.push(Expr::RawTex(format!("\\{command}")));
                    push_visible_items(
                        &mut items,
                        normalize_definition_fallback_expr(self.parse_visible_wrapper_text(&name)?),
                    );
                }
            }
        }
        items.push(Expr::RawTex("}".to_string()));
        Ok(collapse_single_sequence(Expr::Sequence(items)))
    }
}

/// Return true when a raw-only replacement stays entirely on MathType's raw path.
fn replacement_is_pure_raw_tex(items: &[Expr]) -> bool {
    !items.is_empty() && items.iter().all(expr_is_pure_raw_tex_fragment)
}

/// Return true when one rendered replacement fragment carries only raw fallback bytes.
fn expr_is_pure_raw_tex_fragment(expr: &Expr) -> bool {
    match expr {
        Expr::RawTex(_) => true,
        Expr::Sequence(items) => {
            !items.is_empty() && items.iter().all(expr_is_pure_raw_tex_fragment)
        }
        _ => false,
    }
}

/// Return true only when a replacement body is one fully parsed supported environment.
fn is_balanced_supported_environment_body(expr: &Expr) -> bool {
    match expr {
        Expr::Environment { .. } | Expr::Matrix { .. } | Expr::Subarray { .. } => true,
        Expr::Style { content, .. } => is_balanced_supported_environment_body(content),
        Expr::Sequence(items) if items.len() == 1 => {
            is_balanced_supported_environment_body(&items[0])
        }
        _ => false,
    }
}

enum EnvironmentDefinitionFragment {
    Raw(String),
    Visible(String),
    MatchedWrapper {
        leading_ws: String,
        command: String,
        name: String,
    },
}

/// Split environment-definition text so begin/end wrappers stay raw.
fn split_environment_definition_body_fragments(
    content: &str,
) -> Vec<EnvironmentDefinitionFragment> {
    let mut fragments = Vec::new();
    let mut cursor = 0usize;
    while let Some((start, end)) = next_environment_wrapper_span(content, cursor) {
        let raw_start = leading_wrapper_whitespace_start(content, cursor, start);
        if raw_start > cursor {
            fragments.push(EnvironmentDefinitionFragment::Visible(
                content[cursor..raw_start].to_string(),
            ));
        }
        let wrapper = &content[start..end];
        let (command, name) = parse_environment_wrapper(wrapper);
        if environment_wrapper_is_unmatched(content, &command, &name) {
            fragments.push(EnvironmentDefinitionFragment::Raw(
                content[raw_start..end].to_string(),
            ));
        } else {
            fragments.push(EnvironmentDefinitionFragment::MatchedWrapper {
                leading_ws: content[raw_start..start].to_string(),
                command,
                name,
            });
        }
        cursor = end;
    }
    if cursor < content.len() {
        fragments.push(EnvironmentDefinitionFragment::Visible(
            content[cursor..].to_string(),
        ));
    }
    fragments
}

/// Return the next raw `\begin{...}` or `\end{...}` span inside one definition body.
fn next_environment_wrapper_span(content: &str, cursor: usize) -> Option<(usize, usize)> {
    let remainder = &content[cursor..];
    let begin = remainder.find("\\begin{").map(|offset| cursor + offset);
    let end = remainder.find("\\end{").map(|offset| cursor + offset);
    let start = match (begin, end) {
        (Some(left), Some(right)) => left.min(right),
        (Some(left), None) => left,
        (None, Some(right)) => right,
        (None, None) => return None,
    };
    let close = content[start..].find('}')?;
    Some((start, start + close + 1))
}

/// Attach whitespace immediately before a raw environment wrapper to the raw fragment.
fn leading_wrapper_whitespace_start(content: &str, cursor: usize, start: usize) -> usize {
    let mut raw_start = start;
    while raw_start > cursor && content.as_bytes()[raw_start - 1].is_ascii_whitespace() {
        raw_start -= 1;
    }
    raw_start
}

/// Parse one `\begin{...}` or `\end{...}` wrapper into its command and environment name.
fn parse_environment_wrapper(wrapper: &str) -> (String, String) {
    if let Some(name) = wrapper
        .strip_prefix("\\begin{")
        .and_then(|rest| rest.strip_suffix('}'))
    {
        return ("begin".to_string(), name.to_string());
    }
    if let Some(name) = wrapper
        .strip_prefix("\\end{")
        .and_then(|rest| rest.strip_suffix('}'))
    {
        return ("end".to_string(), name.to_string());
    }
    unreachable!("wrapper span must start with \\\\begin{{ or \\\\end{{");
}

/// Return true when this wrapper's environment name is unmatched in the same body.
fn environment_wrapper_is_unmatched(content: &str, command: &str, name: &str) -> bool {
    let begin_count = content.matches(&format!("\\begin{{{name}}}")).count();
    let end_count = content.matches(&format!("\\end{{{name}}}")).count();
    match command {
        "begin" => begin_count > end_count,
        "end" => end_count > begin_count,
        _ => false,
    }
}
