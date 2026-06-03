#!/usr/bin/env python3
"""Probe low-level DOCX replacement of OMML with a MathType OLE object."""

import argparse
import shutil
import sys
from pathlib import Path

if __package__ in (None, ""):
    # Preserve direct execution via `python scripts/mathtype/probe_replace.py`.
    sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
    __package__ = "mathtype"

from .docx_ole import replace_omml_with_template


def copy_for_probe(source: Path, target: Path) -> None:
    """Copy a source file to the probe output path."""
    target.parent.mkdir(parents=True, exist_ok=True)
    shutil.copy2(source, target)


def main() -> int:
    """Run the low-level DOCX MathType OLE replacement probe."""
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--source", default="output/docx/manuscript.docx", help="DOCX containing OMML")
    parser.add_argument("--sample", default="mathtype.docx", help="DOCX containing one MathType OLE object")
    parser.add_argument("--target", default="tmp/mathtype-lowlevel-probe.docx", help="Output probe DOCX")
    parser.add_argument("--limit", type=int, default=1, help="Number of top-level OMML nodes to replace")
    parser.add_argument("--copy-only", action="store_true", help="Only copy the source DOCX")
    args = parser.parse_args()

    source = Path(args.source)
    sample = Path(args.sample)
    target = Path(args.target)

    if args.copy_only:
        copy_for_probe(source, target)
        print(f"[mathtype] copied {source} -> {target}")
        return 0

    replaced = replace_omml_with_template(source, sample, target, args.limit)
    print(f"[mathtype] wrote {target}")
    print(f"[mathtype] replaced top-level OMML nodes: {replaced}")
    return 0 if replaced else 1


if __name__ == "__main__":
    raise SystemExit(main())
