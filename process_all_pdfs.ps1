# PowerShell script to process all PDFs
# This script processes all PDF files in the input folder and generates numbered JSON outputs

Write-Host "Starting PDF processing..." -ForegroundColor Green

# Check if input directory exists
if (-not (Test-Path "input")) {
    Write-Host "Error: input directory not found!" -ForegroundColor Red
    exit 1
}

# Check if output directory exists, create if not
if (-not (Test-Path "output")) {
    New-Item -ItemType Directory -Path "output" -Force | Out-Null
}

# Get all PDF files
$pdfFiles = Get-ChildItem -Path "input" -Filter "*.pdf" | Sort-Object Name
$pdfCount = $pdfFiles.Count

Write-Host "Found $pdfCount PDF files to process" -ForegroundColor Cyan

if ($pdfCount -eq 0) {
    Write-Host "No PDF files found in input directory" -ForegroundColor Yellow
    exit 1
}

# Process each PDF file
foreach ($pdfFile in $pdfFiles) {
    Write-Host "Processing file: $($pdfFile.Name)" -ForegroundColor Yellow
    
    # Generate output filename using the same name as input but with .json extension
    $baseFileName = [System.IO.Path]::GetFileNameWithoutExtension($pdfFile.Name)
    $outputFile = "output\$baseFileName.json"
    
    # Run the adobe1a tool
    $process = Start-Process -FilePath ".\target\release\adobe1a.exe" -ArgumentList "--input", "`"$($pdfFile.FullName)`"", "--output", "`"$outputFile`"" -Wait -PassThru -NoNewWindow
    
    if ($process.ExitCode -eq 0) {
        Write-Host "✅ Successfully processed: $($pdfFile.Name) -> $baseFileName.json" -ForegroundColor Green
    } else {
        Write-Host "❌ Failed to process: $($pdfFile.Name)" -ForegroundColor Red
    }
}

Write-Host "PDF processing completed!" -ForegroundColor Green
Write-Host "Output files are available in output/" -ForegroundColor Cyan
Get-ChildItem -Path "output" -Filter "*.json" | Select-Object Name, Length, LastWriteTime
