# Post-process DOCX file - orchestrator script
# This script calls individual processing scripts in sequence
# Usage: .\postprocess-docx.ps1 -DocxPath "path\to\file.docx"

param(
    [Parameter(Mandatory=$true, HelpMessage="Path to the DOCX file to process")]
    [string]$DocxPath
)

# Color output functions
function Write-Success { param([string]$Message) Write-Host $Message -ForegroundColor Green }
function Write-Info { param([string]$Message) Write-Host $Message -ForegroundColor Cyan }
function Write-Error-Custom { param([string]$Message) Write-Host $Message -ForegroundColor Red }

# Validate inputs
Write-Info "Validating inputs..."
if (-not (Test-Path $DocxPath)) {
    Write-Error-Custom "DOCX file not found: $DocxPath"
    exit 1
}

# Convert to absolute path
$DocxPath = (Resolve-Path $DocxPath).Path
Write-Info "=== Starting DOCX Post-Processing Pipeline ==="
Write-Info "Target file: $DocxPath"
Write-Info ""

# Get script directory
$scriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path

# ===================================================================
# Create Word application and open document (shared across all steps)
# ===================================================================

$word = $null
$doc = $null

try {
    Write-Info "Initializing Word application..."
    $word = New-Object -ComObject Word.Application
    $word.Visible = $false
    $word.DisplayAlerts = 0  # wdAlertsNone
    $word.ScreenUpdating = $false

    Write-Info "Opening document..."
    $doc = $word.Documents.Open($DocxPath)
    Write-Success "Document opened successfully"
    Write-Info ""

# ===================================================================
# Call processing scripts in sequence, sharing Word objects
# ===================================================================
    # 1. Merge table cells based on markers
    Write-Info "Step 1: Merging table cells..."
    $mergeScript = Join-Path $scriptDir "merge-table-cells.ps1"

    if (Test-Path $mergeScript) {
        & $mergeScript -DocxPath $DocxPath -WordApp $word -Document $doc
        if ($LASTEXITCODE -ne 0 -and $null -ne $LASTEXITCODE) {
            throw "merge-table-cells.ps1 failed with exit code $LASTEXITCODE"
        }
        Write-Success "Step 1 completed"
        Write-Info ""
    }
    else {
        Write-Warning "merge-table-cells.ps1 not found, skipping..."
        Write-Info ""
    }

    # 2. Process table metadata from captions
    Write-Info "Step 2: Processing table metadata..."
    $metadataScript = Join-Path $scriptDir "process-table-metadata.ps1"

    if (Test-Path $metadataScript) {
        & $metadataScript -DocxPath $DocxPath -WordApp $word -Document $doc
        if ($LASTEXITCODE -ne 0 -and $null -ne $LASTEXITCODE) {
            throw "process-table-metadata.ps1 failed with exit code $LASTEXITCODE"
        }
        Write-Success "Step 2 completed"
        Write-Info ""
    }
    else {
        Write-Warning "process-table-metadata.ps1 not found, skipping..."
        Write-Info ""
    }

    # 3. Auto-fit tables to window
    Write-Info "Step 3: Auto-fitting tables to window..."
    $autofitScript = Join-Path $scriptDir "autofit-tables.ps1"

    if (Test-Path $autofitScript) {
        & $autofitScript -DocxPath $DocxPath -WordApp $word -Document $doc
        if ($LASTEXITCODE -ne 0 -and $null -ne $LASTEXITCODE) {
            throw "autofit-tables.ps1 failed with exit code $LASTEXITCODE"
        }
        Write-Success "Step 3 completed"
        Write-Info ""
    }
    else {
        Write-Warning "autofit-tables.ps1 not found, skipping..."
        Write-Info ""
    }

    # 4. Add more processing scripts here as needed
    # Example:
    # Write-Info "Step 4: Formatting document..."
    # $formatScript = Join-Path $scriptDir "format-document.ps1"
    # if (Test-Path $formatScript) {
    #     & $formatScript -DocxPath $DocxPath -WordApp $word -Document $doc
    #     if ($LASTEXITCODE -ne 0 -and $null -ne $LASTEXITCODE) {
    #         throw "format-document.ps1 failed with exit code $LASTEXITCODE"
    #     }
    #     Write-Success "Step 3 completed"
    #     Write-Info ""
    # }

    # ===================================================================
    # Save document (all changes from all scripts)
    # ===================================================================
    Write-Info "Saving all changes to document..."
    $doc.Save()
    Write-Success "Document saved successfully"

    Write-Success "=== Post-Processing Pipeline Completed Successfully ==="
}
catch {
    Write-Error-Custom "=== Post-Processing Pipeline Failed ==="
    Write-Error-Custom "Error: $_"
    exit 1
}
finally {
    # ===================================================================
    # Clean up Word COM objects
    # ===================================================================
    Write-Info "Cleaning up Word application..."

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
