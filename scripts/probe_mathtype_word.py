#!/usr/bin/env python3
"""
Probe whether Word + MathType automation can create editable MathType objects.

The script copies a DOCX to a temporary probe file, records package evidence
before and after the Word automation step, and reports whether MathType/OLE
parts appeared. It is intentionally separate from the normal build pipeline.
"""

import argparse
import base64
import shutil
import subprocess
import sys
import zipfile
from dataclasses import dataclass
from pathlib import Path


DEFAULT_SOURCE_DOCX = Path("output/docx/manuscript.docx")
DEFAULT_PROBE_DOCX = Path("tmp/mathtype-probe.docx")
DEFAULT_MATHTYPE_TEMPLATE = Path(
    r"C:\Program Files (x86)\MathType\Office Support\64\MathType Commands 2016.dotm"
)
TEX_TOGGLE_MACRO = "MathTypeCommands.UILib.MTCommand_TeXToggle"
CONVERT_EQUATIONS_MACRO = "MathTypeCommands.UILib.MTCommand_ConvertEqns"
DIRECT_CONVERT_OMML_MACRO = "MTConvertEquations.DoConvertEquations"


@dataclass
class DocxProbeStats:
    """Evidence collected from a DOCX package."""

    omml_count: int
    omml_para_count: int
    ole_relationship_count: int
    object_markup_count: int
    mathtype_marker_count: int
    embedding_part_count: int
    embedding_parts: list[str]

    @property
    def has_mathtype_object_evidence(self) -> bool:
        """Return whether the package looks like it contains MathType/OLE objects."""
        return (
            self.ole_relationship_count > 0
            or self.object_markup_count > 0
            or self.mathtype_marker_count > 0
            or self.embedding_part_count > 0
        )


def print_info(message: str) -> None:
    """Print a probe progress line."""
    print(f"[probe] {message}")


def inspect_docx_package(docx_path: Path) -> DocxProbeStats:
    """Inspect the DOCX zip package for OMML and MathType/OLE evidence."""
    omml_count = 0
    omml_para_count = 0
    ole_relationship_count = 0
    object_markup_count = 0
    mathtype_marker_count = 0
    embedding_parts: list[str] = []

    with zipfile.ZipFile(docx_path) as archive:
        names = archive.namelist()
        embedding_parts = [name for name in names if name.startswith("word/embeddings/")]

        for name in names:
            if not name.endswith((".xml", ".rels")):
                continue
            data = archive.read(name).decode("utf-8", errors="ignore")
            omml_count += data.count("<m:oMath")
            omml_para_count += data.count("<m:oMathPara")
            ole_relationship_count += data.count("/oleObject")
            object_markup_count += data.count("<w:object") + data.count("<o:OLEObject")
            # MathType OLE objects commonly surface through Equation.DSMT markers.
            mathtype_marker_count += data.count("Equation.DSMT") + data.count("MathType")

    return DocxProbeStats(
        omml_count=omml_count,
        omml_para_count=omml_para_count,
        ole_relationship_count=ole_relationship_count,
        object_markup_count=object_markup_count,
        mathtype_marker_count=mathtype_marker_count,
        embedding_part_count=len(embedding_parts),
        embedding_parts=embedding_parts,
    )


def log_stats(label: str, stats: DocxProbeStats) -> None:
    """Print a compact summary of collected DOCX evidence."""
    print_info(
        f"{label}: OMML={stats.omml_count}, OMML paragraphs={stats.omml_para_count}, "
        f"OLE relationships={stats.ole_relationship_count}, object markup={stats.object_markup_count}, "
        f"MathType markers={stats.mathtype_marker_count}, embeddings={stats.embedding_part_count}"
    )
    if stats.embedding_parts:
        print_info(f"{label} embedding parts: {', '.join(stats.embedding_parts[:8])}")


def copy_probe_docx(source: Path, target: Path) -> Path:
    """Copy the source DOCX to a probe path, creating the output directory if needed."""
    if not source.exists():
        raise FileNotFoundError(f"Source DOCX not found: {source}")
    target.parent.mkdir(parents=True, exist_ok=True)
    shutil.copy2(source, target)
    return target


def ps_quote(value: str) -> str:
    """Return a single-quoted PowerShell literal."""
    return "'" + value.replace("'", "''") + "'"


def powershell_script_to_encoded_command(script: str) -> str:
    """Encode a PowerShell script for a safer one-argument invocation."""
    return base64.b64encode(script.encode("utf-16le")).decode("ascii")


def build_word_probe_script(docx_path: Path, template_path: Path, macro_name: str, mode: str, visible: bool) -> str:
    """Build the PowerShell script that drives Word and MathType."""
    visible_literal = "$true" if visible else "$false"
    tex_literal = ps_quote(r"\sqrt{x^2+1}")

    return f"""
$ErrorActionPreference = 'Stop'
$docxPath = {ps_quote(str(docx_path.resolve()))}
$templatePath = {ps_quote(str(template_path))}
$macroName = {ps_quote(macro_name)}
$mode = {ps_quote(mode)}
$word = $null
$doc = $null
try {{
    $word = New-Object -ComObject Word.Application
    $word.Visible = {visible_literal}
    $word.DisplayAlerts = 0
    $word.AutomationSecurity = 1

    if (Test-Path $templatePath) {{
        try {{
            $addIn = $word.AddIns.Add($templatePath, $true)
            $addIn.Installed = $true
        }} catch {{
            Write-Output ('MathType add-in load warning: ' + $_.Exception.Message)
        }}
    }} else {{
        Write-Output ('MathType template not found: ' + $templatePath)
    }}

    $doc = $word.Documents.Open($docxPath, $false, $false)

    if ($mode -eq 'tex-toggle') {{
        # TeX toggle is the smallest low-risk probe for creating a MathType OLE object.
        $selection = $word.Selection
        $selection.EndKey(6) | Out-Null
        $selection.TypeParagraph()
        $selection.TypeText({tex_literal})
        $selection.MoveLeft(1, {len(r"\sqrt{x^2+1}")}, 1) | Out-Null
    }} elseif ($mode -eq 'convert-equations') {{
        # Convert Equations may show MathType's dialog; this mode tests whether it is scriptable.
        $doc.Content.Select()
    }} elseif ($mode -eq 'direct-convert-omml') {{
        # Direct macro entry discovered in MathType's VBA: bit 8 means OMML equations,
        # empty translator means convert to MathType/OLE, and prompt/stat flags are off.
        $doc.Content.Select()
    }} else {{
        throw ('Unknown probe mode: ' + $mode)
    }}

    Write-Output ('Running macro: ' + $macroName)
    if ($mode -eq 'direct-convert-omml') {{
        foreach ($initMacro in @('MathTypeCommands.AutoExec.PrivateMain', 'MathTypeCommands.UILib.IsDLLVersionOK')) {{
            try {{
                Write-Output ('Prewarming MathType macro: ' + $initMacro)
                $word.Run($initMacro) | Out-Null
            }} catch {{
                Write-Output ('Prewarm warning for ' + $initMacro + ': ' + $_.Exception.Message)
            }}
        }}
        $showStats = $false
        $equationTypes = 8
        $selectionOnly = $false
        $promptUser = $false
        $translatorName = ''
        $translatorOptions = 0
        $convertedCount = 0
        # Word.Application.Run exposes its optional arguments as ByRef COM variants in PowerShell.
        $word.Run($macroName, [ref]$showStats, [ref]$equationTypes, [ref]$selectionOnly, [ref]$promptUser, [ref]$translatorName, [ref]$translatorOptions, [ref]$convertedCount) | Out-Null
        Write-Output ('Direct conversion count: ' + $convertedCount)
    }} else {{
        $word.Run($macroName) | Out-Null
    }}
    Start-Sleep -Seconds 2
    $doc.Save()
    Write-Output 'Word automation completed.'
}} finally {{
    if ($doc -ne $null) {{
        try {{ $doc.Close($false) | Out-Null }} catch {{ }}
    }}
    if ($word -ne $null) {{
        try {{ $word.Quit() | Out-Null }} catch {{ }}
    }}
}}
"""


def run_word_macro_probe(
    docx_path: Path,
    template_path: Path,
    macro_name: str,
    mode: str,
    visible: bool,
    timeout_seconds: int,
) -> subprocess.CompletedProcess:
    """Run the Word COM probe through Windows PowerShell."""
    script = build_word_probe_script(docx_path, template_path, macro_name, mode, visible)
    encoded = powershell_script_to_encoded_command(script)
    cmd = [
        "powershell.exe",
        "-NoProfile",
        "-ExecutionPolicy",
        "Bypass",
        "-EncodedCommand",
        encoded,
    ]
    return subprocess.run(
        cmd,
        check=False,
        capture_output=True,
        text=True,
        encoding="utf-8",
        errors="replace",
        timeout=timeout_seconds,
    )


def macro_for_mode(mode: str) -> str:
    """Return the default MathType macro name for a probe mode."""
    if mode == "tex-toggle":
        return TEX_TOGGLE_MACRO
    if mode == "convert-equations":
        return CONVERT_EQUATIONS_MACRO
    if mode == "direct-convert-omml":
        return DIRECT_CONVERT_OMML_MACRO
    raise ValueError(f"Unknown mode: {mode}")


def main() -> int:
    """Run the MathType Word automation probe."""
    parser = argparse.ArgumentParser(
        description="Probe whether Word + MathType can create MathType/OLE objects in a DOCX."
    )
    parser.add_argument("--source", default=str(DEFAULT_SOURCE_DOCX), help="Source DOCX to copy")
    parser.add_argument("--target", default=str(DEFAULT_PROBE_DOCX), help="Probe DOCX output path")
    parser.add_argument(
        "--template",
        default=str(DEFAULT_MATHTYPE_TEMPLATE),
        help="MathType Word add-in template path",
    )
    parser.add_argument(
        "--mode",
        choices=["tex-toggle", "convert-equations", "direct-convert-omml"],
        default="tex-toggle",
        help="MathType macro path to probe",
    )
    parser.add_argument("--macro", help="Override the MathType macro name")
    parser.add_argument("--visible", action="store_true", help="Show Word during automation")
    parser.add_argument("--timeout", type=int, default=30, help="Word automation timeout in seconds")
    parser.add_argument(
        "--inspect-only",
        action="store_true",
        help="Copy and inspect the DOCX without starting Word",
    )
    args = parser.parse_args()

    source = Path(args.source)
    target = Path(args.target)
    template = Path(args.template)
    macro = args.macro or macro_for_mode(args.mode)

    print_info(f"Copying {source} -> {target}")
    copy_probe_docx(source, target)

    before = inspect_docx_package(target)
    log_stats("before", before)
    if args.inspect_only:
        print_info("Inspect-only mode finished without starting Word.")
        return 0 if before.has_mathtype_object_evidence else 1

    print_info(f"Running Word/MathType probe mode '{args.mode}'")
    try:
        result = run_word_macro_probe(
            target,
            template,
            macro,
            args.mode,
            args.visible,
            args.timeout,
        )
    except subprocess.TimeoutExpired:
        print_info("Word automation timed out; MathType likely opened a modal dialog or blocked.")
        return 2

    if result.stdout.strip():
        print(result.stdout.strip())
    if result.stderr.strip():
        print(result.stderr.strip(), file=sys.stderr)
    if result.returncode != 0:
        print_info(f"Word automation failed with exit code {result.returncode}")
        return result.returncode

    after = inspect_docx_package(target)
    log_stats("after", after)

    if after.has_mathtype_object_evidence and not before.has_mathtype_object_evidence:
        print_info("SUCCESS: MathType/OLE evidence appeared in the probe DOCX.")
        return 0

    print_info("No new MathType/OLE evidence appeared in the probe DOCX.")
    return 1


if __name__ == "__main__":
    sys.exit(main())
