# Supported Functions 实现过程记录

这份记录沉淀了本轮 `mathtype-rust` 的工作方法：从针对单个样本的修补，转向
围绕 `docs/Supported Functions.md` 做更系统的覆盖。它主要供后续维护
MathType TeX Input 相关能力时参考。

## 目标

这轮实现的目标不只是让一个复杂压力样本通过。真正的目标是建立一个可重复的覆盖
流程，让 writer 面对新的复杂公式时也更稳：

1. 审计 Supported Functions 语料。
2. 区分原生支持、已确认的 MathType raw fallback、文档语法片段，以及真正未分类
   的 raw fallback。
3. 行为未知时，用 MathType 做探针。
4. 对编码模式相同的命令族生成可复用表。
5. 只有在行为语义明确、且后续可维护时，才添加有针对性的 parser/writer 原生支持。

最重要的规则是：不要回放整条样本公式捕获到的 MTEF body。如果某个功能确实需要
存储字节，也应该存命令级别或生成出来的 reference，而不是存整条公式的 body。

## 当前结果

修正 `--pre-verb 2` 探针路径后，本地 audit 已经达到干净的分类状态：

```text
supported_functions_native_render=862
supported_functions_raw_fallback=82
supported_functions_known_mathtype_raw_fallback=82
supported_functions_unclassified_raw_fallback=0
supported_functions_syntax_fragment=100
supported_functions_parse_error=0

supported_functions_math_native_render=856
supported_functions_math_raw_fallback=64
supported_functions_math_known_mathtype_raw_fallback=64
supported_functions_math_unclassified_raw_fallback=0
supported_functions_math_parse_error=0
```

完整公式视图更能代表真实数学行为。code-span 视图仍然有用，但
`Supported Functions.md` 里有很多条目只是文档片段，例如裸命令、占位符，或没有
匹配结束符的环境开头。

## Audit 流程

添加零散 parser 分支之前，先跑 audit 工具：

```powershell
cargo run --bin audit_supported_functions -- --limit 80
cargo run --bin audit_supported_functions -- --math-only --limit 20
cargo run --bin audit_supported_functions -- --math-only --write-unclassified-jsonl .pmt\probe-manifests\supported-math-unclassified.jsonl
cargo run --bin audit_supported_functions -- --math-only --write-remaining-jsonl .pmt\probe-manifests\supported-math-remaining.jsonl
```

指标含义：

- `native_render`: 没有产生 `Expr::RawTex`，并且成功写出 MTEF。
- `raw_fallback`: 作为 MathType TeX Input 原始文本保留，同时仍然能写出 MTEF。
- `known_mathtype_raw_fallback`: MathType 自己也会把这个命令存成 raw text。
- `unclassified_raw_fallback`: 下一批需要处理的工作队列。
- `syntax_fragment`: 文档语法片段，不是一条独立公式。

只有 `unclassified_raw_fallback` 应被当作实现 backlog。只要 MathType 自己就是
raw text 行为，`raw_fallback` 非零是可以接受的。

## Probe 流程

新的 MathType 探针应使用 `--pre-verb 2`。`--pre-verb 0` 可能会在普通输入上超时，
只适合用来和旧运行结果做对比。

```powershell
cargo run --bin probe_mathtype_tex -- --check-helper --pre-verb 2 --timeout-ms 60000
cargo run --bin probe_mathtype_tex -- --latex "\mathfrak{I}" --pre-verb 2 --timeout-ms 60000
```

探针结论应该记录到 `docs/MathType TeX Input Probe Notes.md`。不要从超时推断
MathType 不支持某个输入。只有已经完成的 helper 输出，或已保存的 reference OLE
文件，才能作为证据。

## 生成策略

生成表放在 `src/generated/` 下，源逻辑在 `src/bin/generate_mtef_tables.rs` 及其
子模块中。命令族很大，或带有命令级别 MathType 字节细节时，应优先使用生成表。

典型例子：

- `\mathcal`、`\mathbb` 和 `\mathfrak` 字体字符表。
- 从 Supported Functions 映射导出的符号别名表和 delimiter 别名表。
- 可见多字符关系写法对应的命令序列别名。
- writer 需要命令特定记录的 sum/operator 命令表。

`\mathfrak` 表很好地说明了为什么需要生成。大多数字母使用显式 Euclid Math One
槽位，但 `\mathfrak{I}` 和 `\mathfrak{R}` 使用特殊的 Symbol-font CHAR 记录。
这一类应该保持 probe/generated，而不是从某个公式里推导。

常用生成命令：

```powershell
cargo run --bin generate_mtef_tables -- --report-existing --only mathfrak_
cargo run --bin generate_mtef_tables -- --only mathfrak_ --output .pmt\mathfrak-char-tables.rs --timeout-ms 60000
cargo run --bin generate_mtef_tables -- --reuse-existing --skip-invalid --timeout-ms 60000
```

主生成文件有意避开 rustfmt，这样表布局更稳定，也更容易看 diff。

## Native 与 Raw 策略

当 writer 可以用通用 parser/writer 规则保留 MathType 可见语义时，适合做原生
支持。本轮例子包括：

- 样式切换：`\displaystyle`、`\textstyle`、`\scriptstyle`、
  `\scriptscriptstyle`。
- 保留内容的 wrapper：HTML wrapper、overlap wrapper、部分 box wrapper，以及直接
  包 math 的 `\vcenter{...}`。
- relation 别名、delimiter 别名、symbol 别名和 text 别名。
- `\underset`、水平 brace/bracket、matrix/layout 环境，以及 TeX infix fraction。
- 已经探出 MathType width byte 的 spacing 别名。

当 MathType TeX Input 自己会把命令词存为 raw text 时，适合保留为 known raw
fallback。本轮例子包括：

- 尚未结构化实现的 extensible arrow 或 sized delimiter。
- `\Large`、`\Huge` 这类字体大小切换。
- `\rule`、`\tag`、`\frak`、`\mathsfit`、`\enspace`。
- `\overgroup` 这类 group/line-segment accent。
- `\r`、`\mathstrut`、`\phantom`、`\hphantom`、`\vphantom`、`\kern`、
  `\mathrel`、`\nonumber`、`\notag` 和 `\hbox`。
- `CD` 这类不支持的环境：MathType 会把 `\begin` 和 `\end` 保留为 raw text，同时
  正常解析中间字符。

只有在 MathType 完成输入后既没有 raw text、也没有可见记录时，才适合把命令当作
被忽略的 layout hint。目前 `\hspace{...}` 和 `\cline{...}` 是这样处理的。

## 已完成的重构

为了避免后续工作继续把最大文件越堆越长，本轮已经做了拆分：

- Parser helper 移到 `src/parser/`。
- MTEF record/encoding helper 移到 `src/mtef/`。
- 生成表逻辑移到 `src/bin/generate_mtef_tables/`。
- Raw fallback 分类集中到 `src/raw_fallback.rs`。
- Audit 和 probe 分别是职责清晰的独立 binary。

以后增加覆盖时，应扩展最接近的现有模块，而不是继续往顶层 parser 或 writer 里
加孤立特例。

## 验证清单

认为一批 Supported Functions 覆盖完成之前，先运行：

```powershell
cargo fmt --check
cargo check --bins
cargo test
cargo run --bin audit_supported_functions -- --limit 20
cargo run --bin audit_supported_functions -- --math-only --limit 20
git diff --check -- scripts/mathtype-rust src/pandoc_manuscript/mathtype/ole_helper/README.md
```

本轮之后的期望状态：

- `cargo test` 通过所有 `mathtype-rust` 测试。
- 两个 audit 视图都有 `unclassified_raw_fallback=0`。
- 两个 audit 视图都有 `parse_error=0`。
- raw fallback render error 保持为 0。
- 剩余所有 raw fallback 都有文档说明，确认是 MathType 已知行为。

Git 关于 CRLF/LF 的行尾提示不是 whitespace-check 失败，但能避免无关行尾 churn 时
还是应该避免。

## 不要再踩的坑

- 不要把整条压力公式的 MTEF body 作为常量塞进代码。
- 不要把 helper 超时归类为 MathType 不支持某个输入。
- 有 generated probe table 可以提供 typeface/font-position 值时，不要猜。
- 不要把文档里的裸命令当作 parser failure。
- 不要把 MathType 已知 raw fallback 强行映射成原生近似，除非这个近似有明确的语义
  策略和聚焦测试。
