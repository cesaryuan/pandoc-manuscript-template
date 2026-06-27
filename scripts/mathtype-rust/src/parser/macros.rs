use super::*;

impl Parser {
    /// Parse a narrow \def/\gdef subset used by Supported Functions examples.
    pub(super) fn parse_macro_definition(&mut self, command: &str) -> Result<Expr, String> {
        self.skip_ws();
        let Some(name) = self.parse_macro_name()? else {
            return Ok(Expr::RawTex(format!("\\{command}")));
        };
        let params = self.parse_macro_parameter_count()?;
        let replacement = self.parse_raw_group("macro replacement")?;
        self.macros.insert(
            name.clone(),
            MacroDefinition {
                params,
                replacement: replacement.clone(),
                render_mode: MacroRenderMode::RawOnly,
            },
        );
        self.render_macro_definition(command, &name, params, &replacement)
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
            expansion_depth: self.expansion_depth + 1,
            pending_raw_ws: false,
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
        let mut args = Vec::new();
        let raw_prefix = match params {
            0 => format!("\\{command}\\{name}"),
            1 => {
                args.push(Expr::Char('1'));
                format!("\\{command}\\{name}#")
            }
            _ => return Ok(Expr::RawTex(format!("\\{command}"))),
        };
        args.extend(render_raw_only_macro_replacement(replacement, self)?);
        Ok(raw_prefix_sequence_with_raw(raw_prefix, args))
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

    /// Parse the supported macro signatures: no parameters or one `#1` parameter.
    pub(super) fn parse_macro_parameter_count(&mut self) -> Result<usize, String> {
        self.skip_ws();
        if self.peek() != Some('#') {
            return Ok(0);
        }
        self.pos += 1;
        if self.peek() != Some('1') {
            return Err("only one-argument macro definitions are supported".to_string());
        }
        self.pos += 1;
        Ok(1)
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

    /// Parse one raw wrapper fragment so unsupported wrapper arguments stay visible in MathType order.
    pub(super) fn parse_visible_wrapper_text(&self, content: &str) -> Result<Expr, String> {
        self.parse_macro_replacement(content)
    }
}
