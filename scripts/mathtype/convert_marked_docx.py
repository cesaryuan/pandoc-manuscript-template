#!/usr/bin/env python3
"""Convert a marker-bearing DOCX from OMML equations to MathType OLE equations."""

import argparse
import sys
from pathlib import Path

if __package__ in (None, ""):
    # Preserve direct execution via `python scripts/mathtype/convert_marked_docx.py`.
    sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
    __package__ = "mathtype"

from logging_utils import log_info

from .marked_docx import extract_marked_equation_requests, inspect_docx, replace_marked_omml_with_generated
from .ole_parts import build_helper, generate_equation_parts


def convert_marked_docx(source: Path, target: Path, work_dir: Path) -> int:
    """Convert all hidden-marker-bound OMML nodes in a DOCX to MathType OLE."""
    requests = extract_marked_equation_requests(source)
    if not requests:
        raise ValueError(
            f"No hidden MathType markers found in {source}; "
            "build the DOCX with scripts/mathtype/mathtype_markers.lua first"
        )

    size_summary = sorted({request.font_size_pt for request in requests if request.font_size_pt is not None})
    log_info(f"[mathtype] marked DOCX math nodes: {len(requests)}")
    if size_summary:
        log_info(f"[mathtype] detected Word font sizes (pt): {', '.join(f'{size:g}' for size in size_summary)}")
    equations = generate_equation_parts(requests, work_dir)
    replaced = replace_marked_omml_with_generated(source, target, equations)
    log_info(f"[mathtype] replaced top-level OMML nodes: {replaced}")
    inspect_docx(target)
    return replaced


def main() -> int:
    """Run marker-bound DOCX conversion to MathType OLE objects."""
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--source", default="tmp/mathtype-marker-source.docx", help="DOCX containing hidden MathType markers")
    parser.add_argument("--target", default="tmp/mathtype-marker-ole-probe.docx", help="Output DOCX")
    parser.add_argument("--mode", choices=["all"], default="all", help="Convert all marker-bound formulas")
    parser.add_argument("--work-dir", default="tmp/mathtype-all", help="Directory for generated OLE and WMF parts")
    args = parser.parse_args()

    build_helper()
    convert_marked_docx(
        source=Path(args.source),
        target=Path(args.target),
        work_dir=Path(args.work_dir),
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
