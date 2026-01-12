# DOCX Post-Processing with PowerShell

This directory contains a PowerShell script for post-processing generated DOCX files using the Word COM API.

## Files

- `postprocess-docx.ps1` - PowerShell script that formats DOCX files directly (no VBA needed)

## How It Works

When you run `make docx`, the build process:

1. Generates DOCX file using Pandoc
2. Automatically runs `postprocess-docx.ps1` (Windows only, if enabled)
3. The PowerShell script:
   - Opens the generated DOCX in Word (invisible mode)
   - Formats tables with consistent styles
   - Updates all fields and cross-references
   - Sets up page properties (A4, margins, etc.)
   - Saves and closes the document

**No VBA required, no security settings to change!**

## Configuration

Edit the top of `Makefile` to enable/disable:

```makefile
# DOCX post-processing (Windows only)
ENABLE_DOCX_POSTPROCESS = true    # Set to false to disable
```

## Current Post-Processing Operations

The script currently performs these operations (see lines 40-108 in `postprocess-docx.ps1`):

### 1. Format All Tables
- Applies "Grid Table 4 - Accent 1" style
- Auto-fits to window width
- Centers tables on page
- Bolds the header row

### 2. Update Fields
- Updates all fields (cross-references, page numbers, etc.)
- Updates table of contents if present

### 3. Setup Page Properties
- Paper size: A4
- Orientation: Portrait
- Margins: 2.5 cm on all sides

### 4. Optional Operations (Commented Out)
You can uncomment these in the script if needed:
- Remove all comments
- Accept all tracked changes

## Customizing Post-Processing

Edit `scripts\postprocess-docx.ps1` and modify the "FORMATTING OPERATIONS" section (lines 40-108).

### Common Customizations

**Change table style:**
```powershell
$table.Style = "Light Grid - Accent 1"  # Or any Word table style name
```

**Different margins:**
```powershell
$marginCm = 3.0  # 3 cm margins
```

**Remove comments automatically:**
```powershell
# Uncomment lines 114-122
if ($doc.Comments.Count -gt 0) {
    $commentCount = $doc.Comments.Count
    while ($doc.Comments.Count -gt 0) {
        $doc.Comments.Item(1).Delete()
    }
    Write-Success "Removed $commentCount comment(s)"
}
```

**Accept all tracked changes:**
```powershell
# Uncomment lines 124-131
if ($doc.Revisions.Count -gt 0) {
    $revisionCount = $doc.Revisions.Count
    $doc.TrackRevisions = $false
    $doc.Revisions.AcceptAll()
    Write-Success "Accepted $revisionCount revision(s)"
}
```

## Advanced Customizations

The script uses the Word COM API (Word.Application object model). You can do anything Word can do:

### Add custom formatting
```powershell
# Example: Set specific font for headings
foreach ($para in $doc.Paragraphs) {
    if ($para.Style -eq "Heading 1") {
        $para.Range.Font.Name = "Arial"
        $para.Range.Font.Size = 16
    }
}
```

### Insert page breaks before specific headings
```powershell
foreach ($para in $doc.Paragraphs) {
    if ($para.Style -eq "Heading 1" -and $para.Range.Start -gt 0) {
        $para.Range.InsertBreak(7)  # wdPageBreak = 7
    }
}
```

### Set different first page header
```powershell
$doc.PageSetup.DifferentFirstPageHeaderFooter = $true
$firstPageHeader = $doc.Sections.Item(1).Headers.Item(1)
$firstPageHeader.Range.Text = "First Page Header"
```

### Find and replace text
```powershell
$word.Selection.Find.ClearFormatting()
$word.Selection.Find.Replacement.ClearFormatting()
$word.Selection.Find.Execute(
    "old text",     # FindText
    $false,         # MatchCase
    $false,         # MatchWholeWord
    $false,         # MatchWildcards
    $false,         # MatchSoundsLike
    $false,         # MatchAllWordForms
    $true,          # Forward
    0,              # Wrap (wdFindContinue)
    $false,         # Format
    "new text",     # ReplaceWith
    2               # Replace (wdReplaceAll)
)
```

## Testing

Test the script independently:

```powershell
.\scripts\postprocess-docx.ps1 -DocxPath "output\docx\manuscript.docx"
```

## Advantages Over VBA

✅ **No security settings needed** - No "Trust access to VBA project" required
✅ **Easier to maintain** - PowerShell is more modern than VBA
✅ **Version control friendly** - Pure text file, no binary .dotm
✅ **Transparent** - All operations visible in the script
✅ **Flexible** - Easy to add conditional logic, logging, etc.

## Requirements

- Windows with Microsoft Word installed
- PowerShell 5.1 or later (included in Windows 10/11)

## Troubleshooting

### Script fails silently

**Solution:** Run the script manually to see detailed error messages:
```powershell
powershell.exe -ExecutionPolicy Bypass -File scripts\postprocess-docx.ps1 -DocxPath "output\docx\manuscript.docx"
```

### Word opens visibly

**Solution:** Make sure `$word.Visible = $false` is set in the script (line 33).

### Changes not saved

**Solution:** Check that `$doc.Save()` is called before closing (line 137).

### Script hangs

**Cause:** Word dialog box waiting for user input.
**Solution:** Ensure `$word.DisplayAlerts = 0` is set (line 34).

## References

- [Word Object Model Reference](https://learn.microsoft.com/en-us/office/vba/api/overview/word/object-model)
- [WdConstants Enumeration](https://learn.microsoft.com/en-us/office/vba/api/word.wdconstants)

## Security Note

The PowerShell script runs with `-ExecutionPolicy Bypass` to avoid execution policy issues. Review the script before running to ensure it's safe.
