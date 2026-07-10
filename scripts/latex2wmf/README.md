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
  --font-size 12
```

`--svg-backend` 可选 `ratex`（默认）或 `typst`。Typst 后端不是直接解析 LaTeX：
它先用 Apache-2.0 许可的 MiTeX 转成 Typst 数学语法，再交给
`typst-as-lib`。RaTeX 后端能从布局结果取得精确 baseline depth；Typst 导出的页面
不保留 inline box baseline，因此 JSON 会把 `baseline_source` 标记为
`typst-0.2em-estimate`。

## Manuscript WMF snapshots

`samples/manuscript` 是 `scripts/mathtype-rust/samples/manuscript` 中 64 个 LaTeX
样本的逐字节副本。测试会先检查两处样本清单和内容一致，再分别通过 RaTeX 与
Typst 后端运行全部样本。成功渲染的快照同时包含最终 WMF 字节和 CLI 实际写出的
JSON metadata，分别存放为 `snapshots/<backend>/eq_*.snap.wmf` 和
`snapshots/<backend>/eq_*_metadata.snap.json`；不使用 SVG 作为快照。当前不支持的
公式保留错误文本快照，因此将来开始支持时也会产生需要审核的变化，并新增真实
WMF/JSON 快照。

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
