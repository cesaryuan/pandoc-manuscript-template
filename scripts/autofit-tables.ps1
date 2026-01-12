# Auto-fit all tables to window width
# This script adjusts all tables to fit window width automatically
# Usage:
#   Standalone: .\autofit-tables.ps1 -DocxPath "path\to\file.docx"
#   With shared objects: .\autofit-tables.ps1 -DocxPath "path" -WordApp $word -Document $doc

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

    Write-Info "Auto-fitting tables to window..."
    $tableCount = $doc.Tables.Count
    $successCount = 0

    if ($tableCount -eq 0) {
        Write-Info "No tables found in document"
    }
    else {
        foreach ($table in $doc.Tables) {
            try {
                # AutoFit to window
                # wdAutoFitFixed = 0 (固定列宽)
                # wdAutoFitContent = 1 (根据内容自动调整)
                # wdAutoFitWindow = 2 (根据窗口自动调整)
                $table.AutoFitBehavior(2)  # wdAutoFitWindow = 2

                # Optionally center align the table
                $table.Rows.Alignment = 1  # wdAlignRowCenter = 1

                $successCount++
            }
            catch {
                Write-Warning "Failed to auto-fit table $successCount : $_"
            }
        }

        Write-Success "Auto-fitted $successCount of $tableCount table(s) to window"
    }

    # Save the document only if we manage the objects
    if ($managedByUs) {
        Write-Info "Saving document..."
        $doc.Save()
        Write-Success "Document saved"
    }

    Write-Success "`nTable auto-fit processing completed successfully!"
}
catch {
    Write-Error "`nTable auto-fit processing failed: $_"
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
