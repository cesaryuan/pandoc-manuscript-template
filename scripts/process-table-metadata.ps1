# Process table metadata from captions
# This script parses metadata in table captions (format: |key=value key2=value2|)
# and applies the settings to tables, then removes the metadata from captions
# Usage:
#   Standalone: .\process-table-metadata.ps1 -DocxPath "path\to\file.docx"
#   With shared objects: .\process-table-metadata.ps1 -DocxPath "path" -WordApp $word -Document $doc

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

# Helper function: Parse metadata from caption text
function Parse-TableMetadata {
    param([string]$captionText)

    $metadata = @{}

    # Match pattern: |key=value key2=value2|
    if ($captionText -match '\|([^|]+)\|\s*$') {
        $metadataString = $matches[1].Trim()

        # Split by spaces and parse key=value pairs
        $pairs = $metadataString -split '\s+'
        foreach ($pair in $pairs) {
            if ($pair -match '^([^=]+)=(.+)$') {
                $key = $matches[1].Trim()
                $value = $matches[2].Trim()
                $metadata[$key] = $value
            }
        }
    }

    return $metadata
}

# Helper function: Remove metadata from caption text
function Remove-MetadataFromCaption {
    param([string]$captionText)

    # Remove the |...| pattern and trim whitespace
    $cleanText = $captionText -replace '\s*\|[^|]+\|\s*', ''
    return $cleanText.Trim()
}

# Helper function: Convert dimension string to points (Word uses points internally)
function Convert-ToPoints {
    param([string]$dimension)

    # Parse dimension with unit (e.g., "0.10cm", "5pt", "0.5in")
    if ($dimension -match '^([\d.]+)(cm|mm|in|pt)$') {
        $value = [double]$matches[1]
        $unit = $matches[2]

        switch ($unit) {
            'cm' { return $value * 28.35 }  # 1 cm = 28.35 points
            'mm' { return $value * 2.835 }  # 1 mm = 2.835 points
            'in' { return $value * 72 }     # 1 inch = 72 points
            'pt' { return $value }          # Already in points
        }
    }

    # Default: assume points if no unit specified
    return [double]$dimension
}

# Helper function: Apply metadata settings to table
function Apply-TableMetadata {
    param(
        $table,
        [hashtable]$metadata
    )

    $appliedSettings = @()

    foreach ($key in $metadata.Keys) {
        $value = $metadata[$key]

        try {
            switch ($key.ToLower()) {
                'cell_margin' {
                    # Set cell margins (all sides)
                    $points = Convert-ToPoints -dimension $value
                    $table.TopPadding = $points
                    $table.BottomPadding = $points
                    $table.LeftPadding = $points
                    $table.RightPadding = $points
                    $appliedSettings += "cell_margin=$value"
                }
                'cell_margin_top' {
                    $points = Convert-ToPoints -dimension $value
                    $table.TopPadding = $points
                    $appliedSettings += "cell_margin_top=$value"
                }
                'cell_margin_bottom' {
                    $points = Convert-ToPoints -dimension $value
                    $table.BottomPadding = $points
                    $appliedSettings += "cell_margin_bottom=$value"
                }
                'cell_margin_left' {
                    $points = Convert-ToPoints -dimension $value
                    $table.LeftPadding = $points
                    $appliedSettings += "cell_margin_left=$value"
                }
                'cell_margin_right' {
                    $points = Convert-ToPoints -dimension $value
                    $table.RightPadding = $points
                    $appliedSettings += "cell_margin_right=$value"
                }
                'cell_spacing' {
                    # Set spacing between cells
                    $points = Convert-ToPoints -dimension $value
                    $table.Spacing = $points
                    $appliedSettings += "cell_spacing=$value"
                }
                'row_height' {
                    # Set row height for all rows
                    $points = Convert-ToPoints -dimension $value
                    foreach ($row in $table.Rows) {
                        $row.Height = $points
                        $row.HeightRule = 0  # wdRowHeightAuto = 0
                    }
                    $appliedSettings += "row_height=$value"
                }
                'alignment' {
                    # Set table alignment (left, center, right)
                    switch ($value.ToLower()) {
                        'left' {
                            $table.Rows.Alignment = 0  # wdAlignRowLeft = 0
                            $appliedSettings += "alignment=left"
                        }
                        'center' {
                            $table.Rows.Alignment = 1  # wdAlignRowCenter = 1
                            $appliedSettings += "alignment=center"
                        }
                        'right' {
                            $table.Rows.Alignment = 2  # wdAlignRowRight = 2
                            $appliedSettings += "alignment=right"
                        }
                    }
                }
                'autofit' {
                    # Set autofit behavior (fixed, content, window)
                    switch ($value.ToLower()) {
                        'fixed' {
                            $table.AutoFitBehavior(0)  # wdAutoFitFixed = 0
                            $appliedSettings += "autofit=fixed"
                        }
                        'content' {
                            $table.AutoFitBehavior(1)  # wdAutoFitContent = 1
                            $appliedSettings += "autofit=content"
                        }
                        'window' {
                            $table.AutoFitBehavior(2)  # wdAutoFitWindow = 2
                            $appliedSettings += "autofit=window"
                        }
                    }
                }
                default {
                    Write-Warning "Unknown metadata key: $key"
                }
            }
        }
        catch {
            Write-Warning "Failed to apply setting $key=$value : $_"
        }
    }

    return $appliedSettings
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

    Write-Info "Processing table metadata from captions..."
    $tableCount = $doc.Tables.Count
    $processedCount = 0
    $totalSettingsApplied = 0

    if ($tableCount -eq 0) {
        Write-Info "No tables found in document"
    }
    else {
        # Iterate through all tables
        for ($i = 1; $i -le $tableCount; $i++) {
            $table = $doc.Tables.Item($i)

            # Get the table's caption (usually in the paragraph after the table)
            # Word tables can have captions before or after
            $captionRange = $null
            $captionText = ""

            # Try to find caption after table
            try {
                # Find the paragraph immediately after the table
                $tableEnd = $table.Range.End
                $foundCaption = $false

                foreach ($para in $doc.Paragraphs) {
                    if ($para.Range.Start -eq $tableEnd) {
                        # This paragraph starts right after the table
                        $styleName = $para.Style.NameLocal
                        if ($styleName -match 'Caption|题注') {
                            $captionRange = $para.Range
                            $captionText = $para.Range.Text
                            $foundCaption = $true
                            break
                        }
                    }
                }
            }
            catch {
                Write-Warning "Cannnot find caption fot this table: $_"
            }

            # Try to find caption before table if not found after
            if (-not $captionRange) {
                try {
                    # Find the paragraph immediately before the table
                    $tableStart = $table.Range.Start

                    foreach ($para in $doc.Paragraphs) {
                        if ($para.Range.End -eq $tableStart) {
                            # This paragraph ends right before the table
                            $styleName = $para.Style.NameLocal
                            if ($styleName -match 'Caption|题注') {
                                $captionRange = $para.Range
                                $captionText = $para.Range.Text
                                break
                            }
                        }
                    }
                }
                catch {
                    Write-Warning "Cannnot find caption fot this table: $_"
                }
            }

            # Process metadata if caption found
            if ($captionRange -and $captionText -match '\|[^|]+=[^|]+\|') {
                Write-Info "Processing Table $i..."

                # Parse metadata
                $metadata = Parse-TableMetadata -captionText $captionText

                if ($metadata.Count -gt 0) {
                    # Apply metadata to table
                    $appliedSettings = Apply-TableMetadata -table $table -metadata $metadata

                    if ($appliedSettings.Count -gt 0) {
                        Write-Success "  Applied: $($appliedSettings -join ', ')"
                        $totalSettingsApplied += $appliedSettings.Count
                    }

                    # Remove metadata from caption using Find/Replace
                    Write-Info "  Original caption: $captionText"
                    $find = $captionRange.Find
                    $find.ClearFormatting()
                    $find.Replacement.ClearFormatting()
                    $find.Text = "\|[!\|]{1,}\|"  # Word wildcard pattern for |...|
                    $find.Replacement.Text = ""
                    $find.Forward = $true
                    $find.Wrap = 0  # wdFindStop
                    $find.Format = $false
                    $find.MatchCase = $false
                    $find.MatchWholeWord = $false
                    $find.MatchWildcards = $true
                    $find.MatchSoundsLike = $false
                    $find.MatchAllWordForms = $false
                    $result = $find.Execute([ref]$find.Text, [ref]$false, [ref]$false, [ref]$true, [ref]$false, [ref]$false, [ref]$true, [ref]0, [ref]$false, [ref]"", [ref]2)  # wdReplaceAll = 2
                    Write-Success "  Metadata removed from caption"

                    $processedCount++
                }
            }
        }

        Write-Success "`nProcessed $processedCount of $tableCount table(s)"
        Write-Success "Total settings applied: $totalSettingsApplied"
    }

    # Save the document only if we manage the objects
    if ($managedByUs) {
        Write-Info "Saving document..."
        $doc.Save()
        Write-Success "Document saved"
    }

    Write-Success "`nTable metadata processing completed successfully!"
}
catch {
    Write-Error "`nTable metadata processing failed: $_"
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
