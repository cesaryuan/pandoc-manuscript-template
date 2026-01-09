# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [1.0.0] - 2025-01-09

### Added
- Initial release of generic Pandoc manuscript template
- Support for DOCX and LaTeX/PDF output formats
- Example manuscript with comprehensive demonstrations of:
  - Citations and cross-references
  - Mathematical equations
  - Tables and figures
  - Multiple journal format examples
- Sample bibliography with diverse entry types
- Pandoc filters for image conversion and resource management
- LaTeX templates for journal formatting
- Make-based build system with configurable targets
- Comprehensive README documentation with:
  - Quick start guide
  - Journal customization examples
  - Troubleshooting section
  - Platform-specific notes
- Example Cursor IDE configurations (archived in `examples/cursor-configs/`)

### Changed
- Converted from paper-specific project to reusable template
- Replaced journal-specific configuration (Wiley) with generic examples for multiple publishers
- Updated output paths from `cacaie/3-latex/` to generic `output/` directory
- Simplified Makefile with configurable variables at top
- Updated `.gitignore` to use generic patterns instead of journal-specific paths

### Removed
- Paper-specific content (road hazard detection research)
- Paper-specific bibliography (moved to `examples/references/paper-specific-example.bib`)
- Revision history directories (`word/`, `revision/`)
- Hardcoded journal paths and build configurations
- Gemini translation command from Makefile

## [Unreleased]

### Planned
- Additional citation style examples
- More journal template examples
- Automated testing for build process
- Docker container for reproducible builds
