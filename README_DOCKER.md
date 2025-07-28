# PDF Processor Docker Setup

This Docker setup processes all PDF files in the `input` folder and generates JSON outputs with the same filename as the input PDFs in the `output` folder.

## Features

- **Font-based heading detection** with confidence scoring
- **Optimized filtering** to avoid false positives
- **Batch processing** of all PDFs in input folder
- **Filename-based output files** for easy identification
- **Cross-platform support** (Docker + native scripts)

## Quick Start

### Using Docker Compose (Recommended)

1. Place your PDF files in the `input` folder
2. Run the processor:
   ```bash
   docker-compose up --build
   ```
3. Check the results in the `output` folder

### Using Docker directly

1. Build the image:
   ```bash
   docker build --platform linux/amd64 -t adobe1a:v1.0 .
   ```

2. Run the container:
   ```bash
   docker run --rm -v $(pwd)/input:/app/input -v $(pwd)/output:/app/output --network none adobe1a:v1.0
   ```

### Windows PowerShell (Native)

1. Build the Rust project:
   ```powershell
   cargo build --release
   ```

2. Run the PowerShell script:
   ```powershell
   .\process_all_pdfs.ps1
   ```

### Linux/Mac (Native)

1. Build the Rust project:
   ```bash
   cargo build --release
   ```

2. Run the bash script:
   ```bash
   chmod +x process_all_pdfs.sh
   ./process_all_pdfs.sh
   ```

## Output Format

Each PDF generates a JSON file with the following structure:

```json
{
  "title": "Document Title",
  "outline": [
    {
      "level": "H1",
      "text": "Chapter 1: Introduction",
      "page": 1,
      "confidence": 0.9
    },
    {
      "level": "H2", 
      "text": "Overview",
      "page": 2,
      "confidence": 0.8
    }
  ]
}
```

## File Naming

- Input: Any PDF files in `input/` folder
- Output: Same filename as input but with `.json` extension (e.g., `document.pdf` → `document.json`)
- Processing order: Alphabetical by filename

## Configuration

The processor includes several optimizations:

- **Font size thresholds**: H1 (>15pt), H2 (12-15pt), H3 (10-12pt)
- **Confidence scoring**: Bold/italic fonts get higher confidence
- **Length filtering**: Text between 4-100 characters
- **Prose detection**: Filters out sentences and paragraph text
- **Top candidates**: Limits to 50 best headings per document

## Troubleshooting

- **No output**: Check that PDF files are in the `input` folder
- **Empty JSON**: PDF might be image-based or have no extractable text
- **Build errors**: Ensure Docker is installed and running
- **Permission errors**: Check folder permissions for input/output directories

## Dependencies

- Rust 1.75+
- Docker (for containerized processing)
- PowerShell (for Windows native processing)
- Bash (for Linux/Mac native processing)
