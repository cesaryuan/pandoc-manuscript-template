# Pandoc Manuscript Template - Makefile
# Generic build system for academic manuscripts

# ============================================================================
# CONFIGURATION - Customize these variables for your project
# ============================================================================

# Project name (used for output files)
PROJECT_NAME = manuscript

# Output directories
OUTPUT_DIR = output
DOCX_DIR = $(OUTPUT_DIR)/docx
LATEX_DIR = $(OUTPUT_DIR)/latex

# Input files
MANUSCRIPT = manuscript.md
IMAGE_FILES = $(wildcard examples/images/*.pdf examples/images/*.png examples/images/*.jpg)

# ============================================================================
# PLATFORM DETECTION - Detect OS and set commands accordingly
# ============================================================================

ifeq ($(OS),Windows_NT)
    # Windows detected - use PowerShell for better compatibility
    SHELL := powershell.exe
    .SHELLFLAGS := -NoProfile -Command
    MKDIR = if (!(Test-Path '$(subst /,\,$(1))')) { New-Item -ItemType Directory -Path '$(subst /,\,$(1))' -Force | Out-Null }
    RM = if (Test-Path '$(subst /,\,$(1))') { Remove-Item -Recurse -Force '$(subst /,\,$(1))' }
    MV = Move-Item -Force '$(subst /,\,$(1))' '$(subst /,\,$(2))'
    ECHO = Write-Host
else
    # Unix/Linux/macOS
    MKDIR = mkdir -p $(1)
    RM = rm -rf $(1)
    MV = mv $(1) $(2)
    ECHO = echo
endif

# ============================================================================
# TARGETS
# ============================================================================

.PHONY: all docx latex pdf clean help

# Default target
all: docx

help:
ifeq ($(OS),Windows_NT)
	@Write-Host "Pandoc Manuscript Template - Available targets:"
	@Write-Host "  make docx      - Generate DOCX file"
	@Write-Host "  make latex     - Generate LaTeX file only"
	@Write-Host "  make pdf       - Generate LaTeX and compile to PDF"
	@Write-Host "  make clean     - Remove all generated files"
	@Write-Host ""
	@Write-Host "Configuration:"
	@Write-Host "  PROJECT_NAME = $(PROJECT_NAME)"
	@Write-Host "  Output: $(OUTPUT_DIR)/"
else
	@echo "Pandoc Manuscript Template - Available targets:"
	@echo "  make docx      - Generate DOCX file"
	@echo "  make latex     - Generate LaTeX file only"
	@echo "  make pdf       - Generate LaTeX and compile to PDF"
	@echo "  make clean     - Remove all generated files"
	@echo ""
	@echo "Configuration:"
	@echo "  PROJECT_NAME = $(PROJECT_NAME)"
	@echo "  Output: $(OUTPUT_DIR)/"
endif

# Generate DOCX file
docx: $(DOCX_DIR)/$(PROJECT_NAME).docx

$(DOCX_DIR)/$(PROJECT_NAME).docx: $(MANUSCRIPT) $(IMAGE_FILES)
ifeq ($(OS),Windows_NT)
	@if (!(Test-Path '$(subst /,\,$(DOCX_DIR))')) { New-Item -ItemType Directory -Path '$(subst /,\,$(DOCX_DIR))' -Force | Out-Null }
	@pandoc --defaults pandoc/pandoc-docx.yml
else
	@mkdir -p $(DOCX_DIR)
	pandoc --defaults pandoc/pandoc-docx.yml
endif

# Generate LaTeX file
latex: $(LATEX_DIR)/$(PROJECT_NAME).tex

$(LATEX_DIR)/$(PROJECT_NAME).tex: $(MANUSCRIPT) $(IMAGE_FILES)
ifeq ($(OS),Windows_NT)
	@if (!(Test-Path '$(subst /,\,$(LATEX_DIR))')) { New-Item -ItemType Directory -Path '$(subst /,\,$(LATEX_DIR))' -Force | Out-Null }
	@pandoc --defaults pandoc/pandoc-latex.yml
else
	@mkdir -p $(LATEX_DIR)
	pandoc --defaults pandoc/pandoc-latex.yml
endif

# Generate PDF from LaTeX (requires LaTeX installation)
pdf: latex
ifeq ($(OS),Windows_NT)
	@Write-Host "Compiling LaTeX to PDF..."
	@Set-Location $(LATEX_DIR); latexmk -interaction=nonstopmode -file-line-error -xelatex -outdir=build $(PROJECT_NAME).tex
	@Move-Item -Force '$(subst /,\,$(LATEX_DIR))\build\$(PROJECT_NAME).pdf' '$(subst /,\,$(OUTPUT_DIR))\$(PROJECT_NAME).pdf'
	@if (Test-Path '$(subst /,\,$(LATEX_DIR))\build') { Remove-Item -Recurse -Force '$(subst /,\,$(LATEX_DIR))\build' }
	@Write-Host "PDF created: $(OUTPUT_DIR)/$(PROJECT_NAME).pdf"
else
	@echo "Compiling LaTeX to PDF..."
	@cd $(LATEX_DIR) && latexmk -interaction=nonstopmode -file-line-error -xelatex -outdir=build $(PROJECT_NAME).tex
	@mv $(LATEX_DIR)/build/$(PROJECT_NAME).pdf $(OUTPUT_DIR)/$(PROJECT_NAME).pdf
	@rm -rf $(LATEX_DIR)/build
	@echo "PDF created: $(OUTPUT_DIR)/$(PROJECT_NAME).pdf"
endif

# Create distribution archive
dist: pdf docx
ifeq ($(OS),Windows_NT)
	@if (!(Test-Path '$(subst /,\,$(OUTPUT_DIR))\dist')) { New-Item -ItemType Directory -Path '$(subst /,\,$(OUTPUT_DIR))\dist' -Force | Out-Null }
	@Set-Location $(LATEX_DIR); Compress-Archive -Force -Path .\* -DestinationPath ..\..\$(OUTPUT_DIR)\dist\$(PROJECT_NAME)-latex.zip
	@Write-Host "Distribution created: $(OUTPUT_DIR)/dist/"
else
	@mkdir -p $(OUTPUT_DIR)/dist
	@cd $(LATEX_DIR) && zip -r ../../$(OUTPUT_DIR)/dist/$(PROJECT_NAME)-latex.zip ./*
	@echo "Distribution created: $(OUTPUT_DIR)/dist/"
endif

# Clean all generated files
clean:
ifeq ($(OS),Windows_NT)
	@Write-Host "Removing generated files..."
	@if (Test-Path '$(subst /,\,$(OUTPUT_DIR))') { Remove-Item -Recurse -Force '$(subst /,\,$(OUTPUT_DIR))' }
	@Write-Host "Clean complete."
else
	@echo "Removing generated files..."
	@rm -rf $(OUTPUT_DIR)
	@echo "Clean complete."
endif

# Deep clean (including pandoc cache)
distclean: clean
ifeq ($(OS),Windows_NT)
	@if (Test-Path '.pandoc-cache') { Remove-Item -Recurse -Force '.pandoc-cache' }
else
	@rm -rf .pandoc-cache
endif
