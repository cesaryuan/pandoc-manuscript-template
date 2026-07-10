# latex2wmf

`latex2wmf` 为 `pmt` 的 MathType DOCX 功能生成跨平台 WMF 公式预览和尺寸 JSON。
MathType OLE/MTEF 仍由相邻的 `mathtype-rust` 生成；这个项目只负责：

1. 使用 `ratex` 直接把 LaTeX 数学公式排版为路径化 SVG；或者
2. 使用 MiTeX 把 LaTeX 数学转换为 Typst，再通过 `typst-as-lib` / `typst-svg`
   生成 SVG；然后
3. 把公式 SVG 中的纯色填充和描边路径转换为 Aldus placeable WMF。

转换器只接受公式渲染会产生的矢量子集。文本必须先转轮廓；位图、透明度、渐变、
滤镜、蒙版和裁剪会明确报错，避免生成在 Word 中静默空白的 WMF。

## Build

```powershell
cargo build --manifest-path scripts\latex2wmf\Cargo.toml
```

## Convert one formula

```powershell
scripts\latex2wmf\target\debug\latex2wmf.exe `
  --latex '$\frac{1}{2}\alpha_i$' `
  --output C:\tmp\formula.wmf `
  --metadata-output C:\tmp\formula.json `
  --svg-output C:\tmp\formula.svg `
  --svg-backend ratex `
  --math-style inline `
  --font-size 12
```

`--svg-backend` 可选 `ratex`（默认）或 `typst`。Typst 后端不是直接解析 LaTeX：
它先用 Apache-2.0 许可的 MiTeX 转成 Typst 数学语法，再交给
`typst-as-lib`。RaTeX 后端直接使用布局 depth；Typst 后端把公式放进带内部 label 的
box，在页面合成丢弃子元素 baseline 之前读取该 box 的真实 `Frame::descent()`，因此
JSON 分别把 `baseline_source` 标记为 `ratex-layout-depth` 或
`typst-frame-baseline+svg-ink-bounds`。Typst 的 frame 表示布局尺寸，但部分斜体字形会
越过 frame；转换器同时测量解析后的 SVG ink bounds，只扩展透明画布，避免 `f` 等
下伸或侧向越界轮廓在 WMF 中被裁掉。

MiTeX 的转换结果会调用 `mitexmathbf`、`mitexarray`、`mitexsqrt` 等其标准数学
scope 中的辅助函数。`latex2wmf` 内嵌与 MiTeX 0.2.4 输出契约对应的精简数学 prelude，
因此不需要在运行时下载 MiTeX Typst 包、WASM 插件或其他 `@preview` 包。纯间距公式
（例如 `\quad`）没有可见路径，但仍会用 XITS 的隐藏 strut 取得真实字体高度，并
生成保留排版宽度的合法空白 WMF。
这类结果的 JSON `baseline_source` 为 `typst-font-strut-baseline`；有可见内容的 Typst
公式仍使用 `typst-frame-baseline+svg-ink-bounds`。

Typst 后端固定使用内嵌的 XITS Math `1.302`，不扫描系统字体，所以不同平台和 wheel
安装后的字形、尺寸及 baseline 保持一致。字体来自官方
[`xits`](https://ctan.org/pkg/xits) CTAN 包，并按 SIL Open Font License 1.1 分发；
来源、校验值、上游说明和许可证保存在 `assets/fonts`。

RaTeX 的布局盒有时会比斜体字形轮廓略紧，例如单个 `b` 的轮廓可能越过零深度
盒子的底边。转换器把 `0.02em` 作为四周的最小安全留白，然后仅扩展透明画布，使
baseline 上方高度、baseline 下方深度和总宽度分别向上落到 `1/2 pt` 网格；公式路径
不会被缩放、裁剪或按墨迹边界 trim。Typst 后端也使用相同的半点画布规则。

因此 JSON 中的宽度、高度和 baseline depth 都是 `1/2 pt` 的整数倍，WMF 的
`1/20 pt` 逻辑边界可以精确表示这些尺寸，且不会因边界向上取整而对横纵方向施加
额外缩放。Word 的 `w:position` 可以直接使用 baseline depth，不附加手工位置偏移。

`--math-style` 可选 `inline` 或 `display`，默认是 `display`。`pmt` 构建 DOCX 时会根据
Pandoc 公式标记自动选择：行内公式使用 RaTeX `Text` 样式，展示公式使用 `Display`
样式。它会影响分式、巨算符和上下限等结构的大小与间距；直接调用 CLI 时应显式传入
与公式所在位置一致的值。

## Manuscript WMF snapshots

`samples/manuscript` 是 `scripts/mathtype-rust/samples/manuscript` 中 64 个 LaTeX
样本的逐字节副本。测试会先检查两处样本清单和内容一致，再分别通过 RaTeX 与
Typst 后端运行全部样本。成功渲染的快照同时包含最终 WMF 字节和 CLI 实际写出的
JSON metadata，分别存放为 `snapshots/<backend>/eq_*.snap.wmf` 和
`snapshots/<backend>/eq_*_metadata.snap.json`；不使用 SVG 作为快照。当前不支持的
公式保留错误文本快照，因此将来开始支持时也会产生需要审核的变化，并新增真实
WMF/JSON 快照。当前 64 个 manuscript 样本在 RaTeX 和 Typst 两个后端均有完整的
WMF/JSON 快照，Typst 目录不再包含错误快照。

`snapshots/ratex-inline` 另外保留一个含分式的行内 RaTeX WMF/JSON 快照，用来检查
`Text` 样式和 `Display` 样式之间的实际渲染差异。`snapshots/typst-inline` 保留
XITS 斜体 `f` 的 WMF/JSON 快照，直接检查超出零 descent frame 的下伸 ink 没有被
WMF 画布裁掉。

普通回归测试：

```powershell
cargo test --manifest-path scripts\latex2wmf\Cargo.toml
```

安装 `cargo-insta` 后，可以集中生成候选快照并逐个接受或拒绝：

```powershell
cargo install cargo-insta
Set-Location scripts\latex2wmf
cargo insta test
cargo insta review
```

快照不一致时，Insta 会保留新的 WMF/JSON 候选文件，并报告发生变化的公式名；只有在
`cargo insta review` 中接受后才会替换基线。若 canonical manuscript 样本发生增删，
同步 `samples/manuscript` 并在 `src/snapshot_tests.rs` 的清单中显式登记，测试会阻止
样本被静默遗漏。
