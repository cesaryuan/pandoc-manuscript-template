# Pandoc Manuscript Template - Makefile
# Wrapper for build.py - provides backward compatibility
#
# This Makefile delegates all build tasks to build.py for better
# maintainability and readability. You can use either:
#   - make <target>  (traditional)
#   - python build.py <target>  (direct)

# ============================================================================
# CONFIGURATION
# ============================================================================

# Build script
BUILD_SCRIPT = scripts/build.py

# ============================================================================
# PLATFORM DETECTION AND RUNNER SELECTION
# ============================================================================

ifeq ($(OS),Windows_NT)
    # Windows - check for uv, fallback to python
    UV_CHECK := $(shell where uv 2>nul)
    ifneq ($(UV_CHECK),)
        RUNNER = uv run
    else
        RUNNER = python
    endif
else
    # Unix/Linux/macOS - check for uv, fallback to python3
    UV_CHECK := $(shell command -v uv 2>/dev/null)
    ifneq ($(UV_CHECK),)
        RUNNER = uv run
    else
        RUNNER = python3
    endif
endif

# ============================================================================
# TARGETS
# ============================================================================

.PHONY: all docx latex clean distclean help

# Default target
all: docx

help:
	@$(RUNNER) $(BUILD_SCRIPT) help

# Generate DOCX file
docx:
	@$(RUNNER) $(BUILD_SCRIPT) docx

# Generate LaTeX file
latex:
	@$(RUNNER) $(BUILD_SCRIPT) latex

# Clean all generated files
clean:
	@$(RUNNER) $(BUILD_SCRIPT) clean

# Deep clean (including pandoc cache)
distclean:
	@$(RUNNER) $(BUILD_SCRIPT) distclean
