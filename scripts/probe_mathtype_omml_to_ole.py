#!/usr/bin/env python3
"""
Probe converting Pandoc OMML equations to MathType OLE objects.

The pipeline is intentionally separate from the normal build:
1. Convert top-level OMML nodes to MathML with Office's OMML2MML.XSL.
2. Replace those OMML nodes with cloned MathType OLE shells.
3. Ask each MathType OLE object to SetMathML through Word COM.
"""

import argparse
import base64
import json
import subprocess
import zipfile
from pathlib import Path

from lxml import etree

from probe_mathtype_docx_ole_replace import replace_omml_with_template


DEFAULT_XSL = Path(r"C:\Program Files\Microsoft Office\root\Office16\OMML2MML.XSL")

NS = {
    "m": "http://schemas.openxmlformats.org/officeDocument/2006/math",
}


def ps_quote(value: str) -> str:
    """Return a single-quoted PowerShell string literal."""
    return "'" + value.replace("'", "''") + "'"


def encode_powershell(script: str) -> str:
    """Encode a PowerShell script for -EncodedCommand."""
    return base64.b64encode(script.encode("utf-16le")).decode("ascii")


def omml_to_mathml_list(docx_path: Path, xsl_path: Path, limit: int) -> list[str]:
    """Extract top-level OMML nodes from a DOCX and convert them to MathML strings."""
    transform = etree.XSLT(etree.parse(str(xsl_path)))
    with zipfile.ZipFile(docx_path) as archive:
        root = etree.fromstring(archive.read("word/document.xml"))

    nodes = root.xpath("//m:oMathPara | //m:oMath[not(parent::m:oMathPara)]", namespaces=NS)
    if limit > 0:
        nodes = nodes[:limit]

    mathml: list[str] = []
    for node in nodes:
        # MathType expects a MathML string, not an XML declaration.
        result_root = transform(node).getroot()
        mathml.append(etree.tostring(result_root, encoding="unicode"))
    return mathml


def build_set_mathml_script(docx_path: Path, mathml_json_path: Path, visible: bool) -> str:
    """Build the PowerShell script that sets MathML on MathType OLE objects."""
    visible_literal = "$true" if visible else "$false"
    return f"""
$ErrorActionPreference = 'Stop'
$docxPath = {ps_quote(str(docx_path.resolve()))}
$mathmlJsonPath = {ps_quote(str(mathml_json_path.resolve()))}
$mathml = Get-Content -LiteralPath $mathmlJsonPath -Raw -Encoding UTF8 | ConvertFrom-Json
$word = $null
$doc = $null
try {{
    $word = New-Object -ComObject Word.Application
    $word.Visible = {visible_literal}
    $word.DisplayAlerts = 0
    $doc = $word.Documents.Open($docxPath, $false, $false)

    $objects = @()
    for ($i = 1; $i -le $doc.InlineShapes.Count; $i++) {{
        $shape = $doc.InlineShapes.Item($i)
        try {{
            if ($shape.OLEFormat.ClassType -eq 'Equation.DSMT4') {{
                $objects += $shape
            }}
        }} catch {{ }}
    }}

    if ($objects.Count -ne $mathml.Count) {{
        throw ('MathType object count mismatch: objects=' + $objects.Count + ', mathml=' + $mathml.Count)
    }}

    for ($i = 0; $i -lt $objects.Count; $i++) {{
        $shape = $objects[$i]
        $shape.OLEFormat.DoVerb(2)
        $obj = $shape.OLEFormat.Object
        try {{
            # SetMathML is a dynamic method exposed by the MathType OLE object.
            $obj.SetMathML([string]$mathml[$i])
        }} finally {{
            try {{ $obj.Close() | Out-Null }} catch {{ }}
        }}
        Write-Output ('set MathML for Equation.DSMT4 object ' + ($i + 1))
    }}

    $doc.Save()
    Write-Output ('saved ' + $docxPath)
}} finally {{
    if ($doc -ne $null) {{
        try {{ $doc.Close($false) | Out-Null }} catch {{ }}
    }}
    if ($word -ne $null) {{
        try {{ $word.Quit() | Out-Null }} catch {{ }}
    }}
}}
"""


def run_set_mathml(docx_path: Path, mathml_json_path: Path, visible: bool, timeout_seconds: int) -> subprocess.CompletedProcess:
    """Run Word COM automation to write MathML into MathType OLE objects."""
    script = build_set_mathml_script(docx_path, mathml_json_path, visible)
    return subprocess.run(
        [
            "powershell.exe",
            "-NoProfile",
            "-ExecutionPolicy",
            "Bypass",
            "-EncodedCommand",
            encode_powershell(script),
        ],
        check=False,
        capture_output=True,
        text=True,
        encoding="utf-8",
        errors="replace",
        timeout=timeout_seconds,
    )


def main() -> int:
    """Run the OMML to MathType OLE probe."""
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--source", default="output/docx/manuscript.docx", help="DOCX containing OMML")
    parser.add_argument("--sample", default="mathtype.docx", help="DOCX containing one MathType OLE object")
    parser.add_argument("--target", default="tmp/mathtype-omml-to-ole-probe.docx", help="Output DOCX")
    parser.add_argument("--xsl", default=str(DEFAULT_XSL), help="Path to Office OMML2MML.XSL")
    parser.add_argument("--limit", type=int, default=0, help="Number of equations to convert; 0 means all")
    parser.add_argument("--visible", action="store_true", help="Show Word while setting MathML")
    parser.add_argument("--timeout", type=int, default=180, help="Word automation timeout in seconds")
    args = parser.parse_args()

    source = Path(args.source)
    sample = Path(args.sample)
    target = Path(args.target)
    xsl = Path(args.xsl)
    limit = args.limit if args.limit > 0 else 10**9

    mathml = omml_to_mathml_list(source, xsl, limit)
    if not mathml:
        print("[probe] no top-level OMML nodes found")
        return 1

    replaced = replace_omml_with_template(source, sample, target, len(mathml))
    print(f"[probe] wrote OLE shell DOCX: {target}")
    print(f"[probe] OMML nodes prepared: {len(mathml)}, replaced: {replaced}")
    if replaced != len(mathml):
        print("[probe] replacement count mismatch before SetMathML")
        return 2

    mathml_json = target.with_suffix(".mathml.json")
    mathml_json.write_text(json.dumps(mathml, ensure_ascii=False), encoding="utf-8")
    result = run_set_mathml(target, mathml_json, args.visible, args.timeout)
    if result.stdout.strip():
        print(result.stdout.strip())
    if result.stderr.strip():
        print(result.stderr.strip())
    if result.returncode != 0:
        print(f"[probe] SetMathML automation failed with exit code {result.returncode}")
        return result.returncode

    print("[probe] MathType SetMathML automation completed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
