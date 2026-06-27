# mathtype-rust

`mathtype-rust` is a native Rust prototype for converting a supported subset of
LaTeX math into MathType-compatible OLE `.bin` files. The generated OLE file
contains an `Equation Native` stream with MTEF content.

当前实现的目标很明确：先覆盖本仓库 `manuscript.md` 中出现的公式，并让生成的
MTEF 与 MathType 通过 `TeX Input Language` 转出来的结果逐字节一致。WMF 预览
暂时不在这个 Rust 程序里生成。

## Build

从本目录运行：

```powershell
cargo build
```

也可以从仓库根目录运行：

```powershell
cargo build --manifest-path scripts\mathtype-rust\Cargo.toml
```

构建后的可执行文件在：

```text
scripts\mathtype-rust\target\debug\mathtype-rust.exe
```

## Developer Verification

开发者最小验收命令：

```powershell
cargo test
cargo fmt -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo run -- --latex '$x$' --output C:\tmp\formula.ole.bin --mtef-output C:\tmp\formula.mtef.bin
```

## Convert One Formula

直接传入 LaTeX 字符串：

```powershell
scripts\mathtype-rust\target\debug\mathtype-rust.exe `
  --latex '$\frac{1}{2}\alpha_i$' `
  --output C:\tmp\formula.ole.bin `
  --mtef-output C:\tmp\formula.mtef.bin
```

从 `.tex` 文件读取公式：

```powershell
scripts\mathtype-rust\target\debug\mathtype-rust.exe `
  --input scripts\mathtype-rust\samples\manuscript\eq_001.tex `
  --output C:\tmp\eq_001.ole.bin `
  --mtef-output C:\tmp\eq_001.mtef.bin
```

参数说明：

- `--latex <tex>`: 直接传入一个公式字符串。
- `--input <file>`: 从文件读取公式。
- `--output <ole.bin>`: 写出 MathType OLE `.bin` 文件。
- `--mtef-output <mtef.bin>`: 可选，额外写出裸 MTEF 字节，方便测试和比对。

`--latex` 和 `--input` 必须二选一。PowerShell 中建议用单引号包住 LaTeX，
避免 `$` 被当成变量展开。

## Generate WMF Preview From MTEF

Rust 程序只负责生成 OLE 和裸 MTEF。若需要 WMF 预览，可以把 `--mtef-output`
写出的裸 MTEF 交给 `MathTypeOleHelper.exe`，让 MathType SDK 执行
`MTEF -> PICT/WMF`：

```powershell
scripts\mathtype-rust\target\debug\mathtype-rust.exe `
  --latex '$$1=a$$' `
  --output C:\tmp\formula.rust.ole.bin `
  --mtef-output C:\tmp\formula.mtef.bin

src\pandoc_manuscript\mathtype\ole_helper\bin\Release\net48\MathTypeOleHelper.exe `
  --method sdk-xform-ole `
  --binary `
  --format "MathType EF" `
  --input C:\tmp\formula.mtef.bin `
  --output C:\tmp\formula.sdk.ole.bin `
  --preview-output C:\tmp\formula.wmf `
  --metadata-output C:\tmp\formula.json
```

这里的 `sdk-xform-ole --binary` 只接受裸 MTEF 输入；不要直接把 LaTeX 文本传给
该方法，因为 MathType 的 `.tdl` 文件是输出 translator，不会把 LaTeX 解析成
公式结构。

## Source Layout

`src` 目录按功能拆分：

- `main.rs`: 二进制入口，只负责挂载模块并调用 CLI。
- `cli.rs`: 命令行参数解析、输入读取、输出写入。
- `ast.rs`: LaTeX 子集解析后使用的表达式 AST。
- `parser.rs`: LaTeX 公式解析和定界符归一化。
- `mtef.rs`: MTEF 字节流生成，包括 MathType 字体定义、模板和符号记录。
- `ole.rs`: 最小 CFB/OLE 容器写入，用于生成 MathType `.ole.bin`。
- `tests.rs`: 递归扫描 `samples`，并与 MathType 参考 OLE 做 byte-for-byte 回归比对。

## Run Tests

运行 Rust 测试：

```powershell
cargo test
```

或从仓库根目录运行：

```powershell
cargo test --manifest-path scripts\mathtype-rust\Cargo.toml
```

`cargo test` 会运行样本回归测试：递归读取 `samples` 下所有子目录里的
`eq_*.tex`，用 Rust 生成 MTEF，再从同一目录中对应的 `mt_eq_*.ole.bin`
读取 `Equation Native` stream，并跳过前 28 字节 OLE native header 后做
byte-for-byte 比对。

当前期望结果是：

```text
test tests::all_samples_match_mathtype_mtef ... ok
```

Rust 默认会捕获通过测试的 stdout。如果想看到样本数量汇总，运行：

```powershell
cargo test -- --show-output
```

期望输出中会包含：

```text
MTEF comparison samples: <passed>/<total> passed
```

如果某个样本不一致，测试会报告样本编号、MathType/Rust MTEF 长度、第一个不同
字节的位置，以及对应的 TeX 内容。

## Manuscript Samples

当前样本目录：

```text
scripts\mathtype-rust\samples\manuscript
```

里面包含：

- `eq_001.tex` 到 `eq_302.tex`: 从 `manuscript.md` 提取出的公式样本。
- `mt_eq_*.ole.bin`: 使用现有 MathType helper 生成的参考 OLE 文件。
- `rust_eq_*.ole.bin`: Rust 程序生成的 OLE 文件。
- `rust_eq_*.mtef.bin`: Rust 程序生成的裸 MTEF 字节。
- `rust_eq_*.err.txt`: 批量运行时捕获的 stderr。

重新批量生成 Rust 输出：

```powershell
$outDir = "scripts\mathtype-rust\samples\manuscript"
$fail = 0
foreach ($f in Get-ChildItem $outDir\eq_*.tex) {
  $base = [System.IO.Path]::GetFileNameWithoutExtension($f.Name)
  $ole = Join-Path $outDir ("rust_" + $base + ".ole.bin")
  $mtef = Join-Path $outDir ("rust_" + $base + ".mtef.bin")
  $errFile = Join-Path $outDir ("rust_" + $base + ".err.txt")

  & scripts\mathtype-rust\target\debug\mathtype-rust.exe `
    --input $f.FullName `
    --output $ole `
    --mtef-output $mtef `
    2> $errFile

  if ($LASTEXITCODE -ne 0) {
    $fail++
  }
}
Write-Output "rust_fail_count=$fail"
```

期望输出：

```text
rust_fail_count=0
```

## Compare Against MathType

下面的命令会读取每个 `mt_eq_*.ole.bin` 的 `Equation Native` stream，跳过
前 28 字节的 OLE native header，然后与 Rust 输出的 `rust_eq_*.mtef.bin`
逐字节比较：

```powershell
python -c "from pathlib import Path; from scripts.mathtype.compound_file import CompoundFile
base=Path('scripts/mathtype-rust/samples/manuscript')
matched=0; mismatched=0; missing=0; examples=[]
for i in range(1,303):
    err=(base/f'rust_eq_{i:03d}.err.txt').read_text(encoding='utf-8', errors='replace') if (base/f'rust_eq_{i:03d}.err.txt').exists() else ''
    if err.startswith('Error:'):
        missing+=1; continue
    rust_path=base/f'rust_eq_{i:03d}.mtef.bin'
    if not rust_path.exists():
        missing+=1; continue
    mt=CompoundFile((base/f'mt_eq_{i:03d}.ole.bin').read_bytes()).read_stream('Equation Native')[28:]
    rust=rust_path.read_bytes()
    if mt==rust:
        matched+=1
    else:
        mismatched+=1
        if len(examples)<30:
            j=next((k for k,(a,b) in enumerate(zip(mt,rust)) if a!=b), min(len(mt),len(rust)))
            tex=(base/f'eq_{i:03d}.tex').read_text(encoding='utf-8').strip()
            examples.append((i,len(mt),len(rust),j,tex))
print('matched',matched,'mismatched',mismatched,'missing',missing)
for row in examples: print(row)"
```

当前期望结果：

```text
matched 302 mismatched 0 missing 0
```

## Supported Functions Audit

`audit_supported_functions` 会扫描
[`docs/Supported Functions.md`](docs/Supported%20Functions.md)，既可以做本地 parser/writer
静态统计，也可以直接调用 MathType helper 做字节级对照。

常用命令：

```powershell
cargo run --bin audit_supported_functions -- --math-only --mathtype-compare --limit 120 --timeout-ms 60000
```

默认输出现在优先保留 `matched`、`mismatched`、`helper_error`、`rust_error`、
`skipped` 这些主结果，不再默认打印 `raw_fallback`、`unclassified_raw_fallback`
以及对应的 examples / sections / command groups，避免和 MathType 对照结果混在一起。

如果确实要排查静态 raw fallback 分类，再显式打开：

```powershell
cargo run --bin audit_supported_functions -- --math-only --show-static-audit
```

## Development Notes

- [`docs/Supported Functions Implementation Log.md`](docs/Supported%20Functions%20Implementation%20Log.md):
  记录本轮从单个复杂样本修补，转向基于 `Supported Functions.md` 的 audit、probe、
  generated-table 和 known raw fallback 工作流。
- [`docs/MathType TeX Input Probe Notes.md`](docs/MathType%20TeX%20Input%20Probe%20Notes.md):
  记录 MathType TeX Input 的实际探针结论，尤其是 native CHAR、raw fallback、
  ignored layout hint 和 helper `--pre-verb 2` 行为。

## Regenerate MathType References

普通使用 Rust 程序不需要 MathType。只有当你修改了样本、更新了 `manuscript.md`
公式，或者想重新确认参考输出时，才需要用现有 helper 调用本机 MathType 生成
`mt_eq_*.ole.bin`。

先确保 helper 已构建：

```powershell
dotnet build src\pandoc_manuscript\mathtype\ole_helper\MathTypeOleHelper.csproj -c Release
```

然后重新生成参考文件：

```powershell
$outDir = "scripts\mathtype-rust\samples\manuscript"
$fail = 0
foreach ($f in Get-ChildItem $outDir\eq_*.tex) {
  $num = [System.IO.Path]::GetFileNameWithoutExtension($f.Name).Substring(3)
  $ole = Join-Path $outDir ("mt_eq_" + $num + ".ole.bin")
  $wmf = Join-Path $outDir ("mt_eq_" + $num + ".wmf")
  $json = Join-Path $outDir ("mt_eq_" + $num + ".json")

  & src\pandoc_manuscript\mathtype\ole_helper\bin\Release\net48\MathTypeOleHelper.exe `
    --method set-data `
    --pre-verb 2 `
    --format 'TeX Input Language' `
    --input $f.FullName `
    --output $ole `
    --encoding utf16le `
    --no-verb `
    --preview-output $wmf `
    --metadata-output $json

  if ($LASTEXITCODE -ne 0) {
    $fail++
  }
}
Write-Output "mathtype_fail_count=$fail"
```

这一步会实际调用本机 MathType，可能比 Rust 转换慢，也更容易受到本机 MathType
安装状态影响。

## Extract Formulas From manuscript.md

如果 `manuscript.md` 的公式发生变化，可以重新提取样本。先用 Pandoc 导出 AST：

```powershell
pandoc manuscript.md -t json -o scripts\mathtype-rust\samples\manuscript_ast.json
```

然后用一个短脚本从 AST 中提取 `Math` 节点，并写成带 `$...$` 或 `$$...$$`
定界符的样本文件。提取后需要重新生成 MathType 参考文件和 Rust 输出，再运行
上面的字节比对。

## Current Scope

当前 parser/writer 是为 manuscript 公式集合收敛出来的精确实现，已经覆盖这些
样本中出现的结构，包括分式、根式、上下标、动态括号、big operators、常用
数学字体、accent、函数名、希腊字母和部分符号。

当前环境类公式分成两种情况：

- `\begin{align}...\end{align}`、`alignat` 等会走原生环境 AST 和 MTEF writer。
- `\begin{aligned}...\end{aligned}` 目前按 MathType TeX Input 的真实行为处理：虽
  然 MathType 能“接受”这段输入，但它不会把 `aligned` 翻译成原生对齐结构，而是
  会保留 `\begin` / `&` / `\end` 这类 raw fallback 片段。
- `\begin{cases}...\end{cases}` 目前仍是受限的已验证路径。

这样可以保证当前样本的 MTEF 与 MathType 完全一致。后续如果要超越“与 MathType
一致”的目标，真正原生支持任意 `aligned` 内容，就需要实现独立于 MathType fallback
的通用环境 writer。
