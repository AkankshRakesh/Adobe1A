#!/bin/bash

# Process all PDFs script
# This script processes all PDF files in the input folder and generates numbered JSON outputs

echo "Starting PDF processing..."

# Check if input directory exists
if [ ! -d "/app/input" ]; then
    echo "Error: /app/input directory not found!"
    exit 1
fi

# Check if output directory exists, create if not
if [ ! -d "/app/output" ]; then
    mkdir -p /app/output
fi

# Count PDF files
pdf_count=$(find /app/input -name "*.pdf" -type f | wc -l)
echo "Found $pdf_count PDF files to process"

if [ $pdf_count -eq 0 ]; then
    echo "No PDF files found in /app/input directory"
    exit 1
fi

# Count PDF files
pdf_count=$(find /app/input -name "*.pdf" -type f | wc -l)
echo "Found $pdf_count PDF files to process"

if [ $pdf_count -eq 0 ]; then
    echo "No PDF files found in /app/input directory"
    exit 1
fi

# Process each PDF file
find /app/input -name "*.pdf" -type f | sort | while read pdf_file; do
    echo "Processing file: $(basename "$pdf_file")"
    
    # Extract base filename without extension
    base_filename=$(basename "$pdf_file" .pdf)
    
    # Generate output filename using the same name as input but with .json extension
    output_file="/app/output/${base_filename}.json"
    
    # Run the adobe1a tool
    if adobe1a -i "$pdf_file" -o "$output_file"; then
        echo "✅ Successfully processed: $(basename "$pdf_file") -> ${base_filename}.json"
    else
        echo "❌ Failed to process: $(basename "$pdf_file")"
    fi
done

echo "PDF processing completed!"
echo "Output files are available in /app/output/"
ls -la /app/output/
