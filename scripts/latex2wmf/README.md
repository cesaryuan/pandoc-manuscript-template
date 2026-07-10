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

