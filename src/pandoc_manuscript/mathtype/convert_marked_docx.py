#!/usr/bin/env python3
"""Convert a marker-bearing DOCX from OMML equations to MathType OLE equations."""

import argparse
from pathlib import Path
from typing import Any

from ..runtime.logging import log_info
from ..runtime.paths import PMT_MATHTYPE_WORK_DIR

from .marked_docx import extract_marked_equation_requests, inspect_docx, replace_marked_omml_with_generated
from .ole_parts import build_helper, generate_equation_parts, normalize_conversion_method


def convert_marked_docx(source: Path, target: Path, work_dir: Path, metadata: dict[str, Any] | None = None) -> int:
    """Convert all hidden-marker-bound OMML nodes in a DOCX to MathType OLE."""
    requests = extract_marked_equation_requests(source)
    if not requests:
        raise ValueError(
            f"No hidden MathType markers found in {source}; "
            "build the DOCX with the packaged MathType marker Lua filter first"
        )

    size_summary = sorted({request.font_size_pt for request in requests if request.font_size_pt is not None})
    log_info(f"[mathtype] marked DOCX math nodes: {len(requests)}")
    if size_summary:
        log_info(f"[mathtype] detected Word font sizes (pt): {', '.join(f'{size:g}' for size in size_summary)}")
    conversion_method = normalize_conversion_method((metadata or {}).get("mathtypeConversionMethod"))
    log_info(f"[mathtype] conversion method: {conversion_method}")
    equations = generate_equation_parts(requests, work_dir, conversion_method=conversion_method)
    replaced = replace_marked_omml_with_generated(source, target, equations)
    log_info(f"[mathtype] replaced top-level OMML nodes: {replaced}")
    inspect_docx(target)
    return replaced


def main() -> int:
    """Run marker-bound DOCX conversion to MathType OLE objects."""
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--source", default=str(PMT_MATHTYPE_WORK_DIR / "marker-source.docx"), help="DOCX containing hidden MathType markers")
    parser.add_argument("--target", default=str(PMT_MATHTYPE_WORK_DIR / "marker-ole-probe.docx"), help="Output DOCX")
    parser.add_argument("--mode", choices=["all"], default="all", help="Convert all marker-bound formulas")
    parser.add_argument("--work-dir", default=str(PMT_MATHTYPE_WORK_DIR / "all"), help="Directory for generated OLE and WMF parts")
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
