# MathType OLE Helper

`MathTypeOleHelper.exe` 是本仓库在 Windows 上调用 MathType 的小工具，用来生成
MathType OLE `.bin`，并可选生成 WMF 预览和预览尺寸 metadata。

当前只保留两个可用方法：

- `set-data`: 通过 MathType OLE COM 对象导入 TeX/LaTeX 文本。
- `sdk-xform-ole`: 接收裸 MTEF 字节，包装成 MathType OLE，并用 MathType SDK 生成 WMF。

## Build

从仓库根目录运行：

```powershell
dotnet build src\pandoc_manuscript\mathtype\ole_helper\MathTypeOleHelper.csproj -c Release
```

构建后的程序在：

```text
src\pandoc_manuscript\mathtype\ole_helper\bin\Release\net48\MathTypeOleHelper.exe
```

建议先在 PowerShell 中保存路径，后面的命令会短很多：

```powershell
$helper = "src\pandoc_manuscript\mathtype\ole_helper\bin\Release\net48\MathTypeOleHelper.exe"
```

## Usage

```text
MathTypeOleHelper --format <clipboard format> (--input <file-or-tex-or-mtef> --output <ole.bin> | --batch-manifest <jobs.json>) [--encoding utf8|utf16le] [--binary] [--no-verb] [--method set-data|sdk-xform-ole] [--pre-verb 2] [--prefs-file <eqp>] [--preview-output <wmf>] [--metadata-output <json>]
```

参数说明：

- `--method <name>`: 转换方法。默认是 `set-data`，可选 `set-data` 或 `sdk-xform-ole`。
- `--format <name>`: Windows clipboard format 名称。`set-data` 常用 `"TeX Input Language"`；`sdk-xform-ole --binary` 建议写 `"MathType EF"`。
- `--input <value>`: 输入。可以是文件路径，也可以是 literal TeX 文本；`--binary` 时必须是文件路径。
- `--output <ole.bin>`: 输出 MathType OLE `.bin`。
- `--batch-manifest <jobs.json>`: 批量模式入口。JSON 中为每条公式单独提供 `input`、`output`，并可选 `previewOutput`、`metadataOutput`。
- `--encoding <name>`: 文本输入编码，可选 `utf8` 或 `utf16le`。默认 `utf8`。
- `--binary`: 按二进制文件读取 `--input`。`sdk-xform-ole` 必须使用这个参数。
- `--no-verb`: 跳过最终的 `DoVerb(2)`。
- `--pre-verb 2`: 在写入公式数据前先调用一次 `IOleObject.DoVerb(2)`。这是当前仓库唯一允许的 pre-open verb 值。
- `--prefs-file <eqp>`: 应用 MathType `.eqp` 偏好文件。`set-data` 会应用到新建公式；`sdk-xform-ole` 会应用到 WMF 预览 transform。
- `--preview-output <wmf>`: 额外输出 WMF 预览。
- `--metadata-output <json>`: 额外输出 WMF 尺寸和 MathType baseline metadata。只有同时写出预览时才会写 metadata。

## Batch Mode

`--batch-manifest` 用来一次性处理多条公式，避免每条公式都单独启动一个 helper 进程。
批量模式下，`--format`、`--method`、`--encoding`、`--binary`、`--no-verb`、`--pre-verb`
和 `--prefs-file` 这些共享参数仍然从命令行传入；每条任务自己的输入输出路径写在
manifest 里。

manifest 示例：

```json
[
  {
    "input": "scripts/mathtype-rust/samples/generated/eq_029.tex",
    "output": ".pmt/eq_029.ole.bin",
    "previewOutput": ".pmt/eq_029.wmf",
    "metadataOutput": ".pmt/eq_029.json"
  },
  {
    "input": "scripts/mathtype-rust/samples/generated/eq_030.tex",
    "output": ".pmt/eq_030.ole.bin"
  }
]
```

调用示例：

```powershell
& $helper `
  --method set-data `
  --pre-verb 2 `
  --format "TeX Input Language" `
  --batch-manifest .pmt\mathtype-batch.json `
  --encoding utf16le `
  --no-verb
```

注意：

- 批量模式下不要再同时传 `--input`、`--output`、`--preview-output`、`--metadata-output`。
- manifest 每条任务都必须有 `input` 和 `output`。
- `previewOutput` / `metadataOutput` 是逐条任务可选的，不是全局参数。
- helper 会在每条批量任务结束后主动回收本次拉起的 MathType 窗口，避免窗口越积越多。

## set-data

`set-data` 走 MathType OLE COM 路线：先创建 `Equation.DSMT4` OLE 对象，再通过
`IDataObject.SetData` 写入指定 clipboard format 的数据，最后保存为 OLE `.bin`。

这个方法适合直接把 LaTeX/TeX 文本交给 MathType：

```powershell
& $helper `
  --method set-data `
  --pre-verb 2 `
  --format "TeX Input Language" `
  --input '$$1=a$$' `
  --output .pmt-eq.ole.bin `
  --encoding utf16le `
  --no-verb `
  --preview-output .pmt-eq.wmf `
  --metadata-output .pmt-eq.json
```

常用约定：

- `--pre-verb 2` 会在写入数据前打开 MathType OLE 对象；这里只支持 `2`，传其他值会直接报错。
- `--no-verb` 会跳过写入后的 `DoVerb(2)`，用于避免某些机器在二次 verb 时崩溃。
- `--encoding utf16le` 推荐配合 `"TeX Input Language"` 使用。
- `--input` 如果不是已存在文件路径，就会按 literal TeX 文本处理。

如果要从文件读取公式：

```powershell
& $helper `
  --method set-data `
  --pre-verb 2 `
  --format "TeX Input Language" `
  --input C:\tmp\formula.tex `
  --output C:\tmp\formula.ole.bin `
  --encoding utf16le `
  --no-verb
```

如果要应用 MathType 偏好文件：

```powershell
& $helper `
  --method set-data `
  --pre-verb 2 `
  --format "TeX Input Language" `
  --input '$$1=a$$' `
  --output C:\tmp\formula.ole.bin `
  --encoding utf16le `
  --prefs-file C:\tmp\math-size.eqp `
  --no-verb
```

运行要求：

- Windows。
- 已安装并注册 MathType OLE COM server，代码中使用的 ProgID 是 `Equation.DSMT4`。
- 如果使用 `--prefs-file` 或 metadata 中的 MathType dimension 信息，还需要能找到 MathType 的 `MT6.dll`。

## sdk-xform-ole

`sdk-xform-ole` 只接受裸 MTEF 字节。它不会把 LaTeX 文本解析成公式，也不会调用
OLE `SetData`/`DoVerb`。当前流程是：

1. 读取 `--input` 指向的裸 MTEF 文件。
2. 使用 `MathTypeSDK.getOLEBase64(mtef)` 包装成 MathType OLE `.bin`。
3. 如果传入 `--preview-output`，再用 MathType SDK 的 `MTXFormEqn` 把 MTEF 转成 WMF。
4. 如果传入 `--metadata-output`，从 WMF 和 SDK 返回的尺寸中写出 JSON metadata。

典型用法是先让 `scripts\mathtype-rust` 生成裸 MTEF，再交给 helper 生成 WMF：

```powershell
scripts\mathtype-rust\target\debug\mathtype-rust.exe `
  --latex '$$1=a$$' `
  --output .pmt-rust.ole.bin `
  --mtef-output .pmt-rust.mtef.bin

& $helper `
  --method sdk-xform-ole `
  --binary `
  --format "MathType EF" `
  --input .pmt-rust.mtef.bin `
  --output .pmt-rust-sdk.ole.bin `
  --preview-output .pmt-rust-sdk.wmf `
  --metadata-output .pmt-rust-sdk.json
```

注意事项：

- `--binary` 是必需的，否则会直接报错。
- `--input` 必须是裸 MTEF，不是 OLE `.bin`，也不是带 28 字节 OLE native header 的 `Equation Native` stream。
- `--format` 目前仍是通用参数解析所需；在 `sdk-xform-ole --binary` 中不参与 MTEF 解析，建议按惯例写 `"MathType EF"`。
- 生成 OLE `.bin` 不走 OLE COM server；生成 WMF 预览需要 MathType SDK 和 `MT6.dll`。
- `--prefs-file` 会通过 `MTXFormSetPrefs` 应用到下一次 `MTXFormEqn`，因此只影响 `--preview-output` 和 `--metadata-output` 对应的 WMF 排版结果，不会改写输入 MTEF 或输出 OLE 内部公式数据。

带偏好文件生成 WMF：

```powershell
& $helper `
  --method sdk-xform-ole `
  --binary `
  --format "MathType EF" `
  --input .pmt-rust.mtef.bin `
  --output .pmt-rust-sdk.ole.bin `
  --preview-output .pmt-rust-sdk.wmf `
  --metadata-output .pmt-rust-sdk.json `
  --prefs-file C:\tmp\math-size.eqp
```

## Metadata JSON

传入 `--preview-output` 和 `--metadata-output` 后，会写出一个很小的 JSON 文件，供
Python 侧写 Word XML 时使用。示例结构如下：

```json
{
  "map_mode": 8,
  "x_ext": 123,
  "y_ext": 45,
  "units_per_inch": 1440,
  "width_pt": 6.1500,
  "height_pt": 2.2500,
  "mathtype": {
    "width_raw": 197,
    "height_raw": 72,
    "baseline_from_bottom_raw": 32,
    "width_pt": 6.1563,
    "height_pt": 2.2500,
    "baseline_from_bottom_pt": 1.0000,
    "horiz_pos_type": 0,
    "horiz_pos": 0
  }
}
```

字段说明：

- `width_pt` / `height_pt`: 由 WMF 尺寸换算出的 point 单位尺寸。
- `mathtype`: MathType 自身的尺寸信息；如果无法读取则为 `null`。
- `baseline_from_bottom_pt`: baseline 到公式框底部的距离，单位 point。
- `baseline_from_bottom_raw`: MathType 内部 1/32 point 单位。

`set-data` 会优先使用 MathType API 的 `MTGetLastDimension` 读取尺寸；`sdk-xform-ole`
会从 WMF 中的 MathType `MFCOMMENT` baseline 信息补齐 baseline。

## Diagnostics

开启详细日志：

```powershell
$env:MATHTYPE_OLE_HELPER_VERBOSE = "1"
& $helper --method set-data --pre-verb 2 --format "TeX Input Language" --input '$$1=a$$' --output .pmt-eq.ole.bin --encoding utf16le --no-verb
```

日志会输出到 stderr，前缀是 `[ole-helper]`。程序成功时退出码为 `0`，失败时退出码为
`1`。如果遇到 COM 或 .NET 异常，verbose 模式会逐项输出异常类型、HResult、message
和 stack trace，避免 `Exception.ToString()` 自身失败时吞掉真正错误。

## Unsupported Methods

下面这些早期实验方法已经不再支持：

- `create-from-data`
- `init-from-data`
- `set-data-cold`
- `sdk-data-object`
- `sdk-list-translators`

另外，`sdk-xform-ole` 的文本输入模式也不支持。原因是 MathType 的 `.tdl`
translator 主要用于把已有 MTEF 公式导出为 TeX/LaTeX 等文本格式，不是 LaTeX
导入解析器；直接把 LaTeX 文本交给 SDK transform 会得到接近普通文本的结果，而不是
MathType 公式结构。
