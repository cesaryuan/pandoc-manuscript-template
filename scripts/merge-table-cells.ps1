# Merge table cells based on markers (!<-! for left merge, !^! for up merge)
# This script implements the VBA MergeCellsLeftAndUp functionality
# Usage:
#   Standalone: .\merge-table-cells.ps1 -DocxPath "path\to\file.docx"
#   With shared objects: .\merge-table-cells.ps1 -DocxPath "path" -WordApp $word -Document $doc

param(
    [Parameter(Mandatory=$true, HelpMessage="Path to the DOCX file to process")]
    [string]$DocxPath,

    [Parameter(Mandatory=$false, HelpMessage="Existing Word Application COM object (optional)")]
    $WordApp = $null,

    [Parameter(Mandatory=$false, HelpMessage="Existing Word Document object (optional)")]
    $Document = $null
)

# Color output functions
function Write-Success { param([string]$Message) Write-Host $Message -ForegroundColor Green }
function Write-Info { param([string]$Message) Write-Host $Message -ForegroundColor Cyan }
function Write-Warning { param([string]$Message) Write-Host $Message -ForegroundColor Yellow }

# Helper function: Clean cell text (remove trailing hidden characters)
function Get-CleanCellText {
    param([string]$text)

    if ($text.Length -ge 2) {
        return $text.Substring(0, $text.Length - 2)
    }
    return $text
}

# Validate inputs
Write-Info "Validating inputs..."
if (-not (Test-Path $DocxPath)) {
    Write-Error "DOCX file not found: $DocxPath"
    exit 1
}

# Convert to absolute path
$DocxPath = (Resolve-Path $DocxPath).Path
Write-Info "Processing: $DocxPath"

# Determine if we need to manage Word objects ourselves
$managedByUs = ($null -eq $WordApp -or $null -eq $Document)

$word = $null
$doc = $null

try {
    if ($managedByUs) {
        # Create Word application
        Write-Info "Starting Word application..."
        $word = New-Object -ComObject Word.Application
        $word.Visible = $false
        $word.DisplayAlerts = 0  # wdAlertsNone
        $word.ScreenUpdating = $false

        # Open document
        Write-Info "Opening document..."
        $doc = $word.Documents.Open($DocxPath)
    }
    else {
        # Use provided objects
        Write-Info "Using shared Word application and document objects..."
        $word = $WordApp
        $doc = $Document
    }

    Write-Info "Processing table cell merges..."
    $tableCount = $doc.Tables.Count
    $leftMergeCount = 0
    $upMergeCount = 0

    if ($tableCount -eq 0) {
        Write-Info "No tables found in document"
    }
    else {
        foreach ($table in $doc.Tables) {
            # ===================================================================
            # Phase 1: Process left merges (!<-!)
            # Iterate from last row to first, last column to second column
            # ===================================================================

            for ($r = $table.Rows.Count; $r -ge 1; $r--) {
                $row = $table.Rows.Item($r)

                # Iterate from last cell to second cell (reverse order)
                for ($c = $row.Cells.Count; $c -ge 2; $c--) {
                    try {
                        $cell = $row.Cells.Item($c)
                        $cellText = Get-CleanCellText -text $cell.Range.Text

                        if ($cellText -eq "!<!") {
                            # Clear the marker text
                            $cell.Range.Text = ""

                            # Merge with left cell
                            try {
                                $leftCell = $row.Cells.Item($c - 1)
                                $leftCell.Merge($cell)
                                $leftMergeCount++
                            }
                            catch {
                                Write-Warning "Failed to merge left at row $r, cell $c : $_"
                            }
                        }
                    }
                    catch {
                        # Skip cells that cause errors (might be part of merged cells)
                        continue
                    }
                }
            }

            # ===================================================================
            # Phase 2: Process up merges (!^!)
            # Iterate from last row to second row
            # ===================================================================

            for ($r = $table.Rows.Count; $r -ge 2; $r--) {
                $row = $table.Rows.Item($r)

                # Check each cell in current row
                for ($c = $row.Cells.Count; $c -ge 1; $c--) {
                    try {
                        $cell = $row.Cells.Item($c)
                        $cellText = Get-CleanCellText -text $cell.Range.Text

                        if ($cellText -eq "!^!") {
                            # Clear the marker text
                            $cell.Range.Text = ""

                            # Merge with cell above
                            try {
                                $aboveCell = $table.Cell($r - 1, $c)
                                $aboveCell.Merge($cell)
                                $upMergeCount++
                            }
                            catch {
                                Write-Warning "Failed to merge up at row $r, cell $c : $_"
                            }
                        }
                    }
                    catch {
                        # Skip cells that cause errors (might be part of merged cells)
                        continue
                    }
                }
            }
        }

        Write-Success "Processed $tableCount table(s)"
        Write-Success "Left merges: $leftMergeCount"
        Write-Success "Up merges: $upMergeCount"
    }

    # Save the document only if we manage the objects
    if ($managedByUs) {
        Write-Info "Saving document..."
        $doc.Save()
        Write-Success "Document saved"
    }

    Write-Success "`nTable cell merge processing completed successfully!"
}
catch {
    Write-Error "`nTable cell merge processing failed: $_"
    Write-Error $_.ScriptStackTrace
    exit 1
}
finally {
    # Clean up COM objects only if we created them
    if ($managedByUs) {
        Write-Info "Cleaning up..."

        if ($doc) {
            $doc.Close([ref]$false)
            [System.Runtime.Interopservices.Marshal]::ReleaseComObject($doc) | Out-Null
        }

        if ($word) {
            $word.Quit()
            [System.Runtime.Interopservices.Marshal]::ReleaseComObject($word) | Out-Null
        }

        # Force garbage collection
        [System.GC]::Collect()
        [System.GC]::WaitForPendingFinalizers()
        [System.GC]::Collect()

        Write-Info "Cleanup complete"
    }
}
