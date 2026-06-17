# Pandoc Manuscript Template - Makefile
# Wrapper for pmt - provides backward-compatible make targets
#
# This Makefile delegates all build tasks to the installed pmt CLI so make and
# direct CLI usage exercise the same package-owned build pipeline. You can use:
#   - make <target>  (traditional)
#   - pmt build <target>  (direct)

# ============================================================================
# CONFIGURATION
# ============================================================================

# CLI executable. Override with `make docx PMT="uv run pmt"` if needed.
PMT ?= pmt

# ============================================================================
# TARGETS
# ============================================================================

.PHONY: all docx latex json clean distclean help

# Default target
all: docx

help:
	@$(PMT) build help

# Generate DOCX file
docx:
	@$(PMT) build docx

# Generate LaTeX file
latex:
	@$(PMT) build latex

# Generate Pandoc JSON AST for debugging
json:
	@$(PMT) build json

# Clean all generated files
clean:
	@$(PMT) build clean

# Deep clean (including pandoc cache)
distclean:
	@$(PMT) build distclean
