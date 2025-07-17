# PDF Outline Extractor - Adobe Hackathon Round 1A

## Goal
Extract a clean, structured outline (title + hierarchical headings H1–H3 with page numbers) from any PDF (≤50 pages) **offline** in under 10 seconds on CPU-only systems with multilingual support.

```json
{
  "title": "Sample Title",
  "outline": [
    { "level": "H1", "text": "Heading Text", "page": 1 },
    ...
  ]
}
```

## Approach
Our solution combines multiple heuristics for robust heading detection:

1. **Primary Parser**: PyMuPDF (fitz) for superior accuracy and multilingual support
2. **Fallback Parser**: pdfminer.six for compatibility
3. **Font Analysis**: Size ranking and boldness detection
4. **Pattern Recognition**: Numeric heading patterns (1., 2.1, 3.1.2, etc.)
5. **Content Filtering**: Excludes dates, pure numbers, and non-heading text
6. **Text Normalization**: Handles various text cases and formatting
7. **Multilingual Support**: Unicode-aware processing for international documents

### Libraries Used
- **PyMuPDF (fitz)**: Primary PDF parsing library (~15MB)
- **pdfminer.six**: Fallback PDF parser (~6MB)
- **Standard Python libraries**: json, re, pathlib, collections

Total package size: ~25MB (well under 200MB limit)

## Directory Layout
```
/app
 ├─ extractor/             # reusable extraction module
 │   ├─ __init__.py
 │   └─ pdf_outline_extractor.py  # main extraction logic
 ├─ input/                 # PDF files to process (created automatically)
 ├─ output/                # Generated JSON files (cleared on each run)
 ├─ main.py                # container entry-point
 ├─ requirements.txt       # Python dependencies
 ├─ Dockerfile            # AMD64-compatible container
 └─ README.md             # this file
```

## Build & Run

### Docker (Production)
```bash
# Build for AMD64 architecture
docker build --platform linux/amd64 -t mysolutionname:somerandomidentifier .

# Run with volume mounts (no network access)
docker run --rm -v $(pwd)/input:/app/input -v $(pwd)/output:/app/output --network none mysolutionname:somerandomidentifier
```

### Local Development
```bash
# Setup virtual environment
python -m venv .venv
source .venv/bin/activate  # On Windows: .venv\Scripts\activate

# Install dependencies
pip install -r requirements.txt

# Run locally
python main.py
```

### Usage
1. Place PDF files (≤50 pages) in the `input/` directory
2. Run the container or script
3. JSON outline files will appear in `output/` with matching filenames
4. Output directory is automatically cleared on each run

## Performance & Constraints

### Execution Time
- ≤10 seconds for 50-page PDF (typically 2-4 seconds)
- Tested on AMD64 systems with 8 CPUs and 16GB RAM

### Resource Requirements
- CPU-only execution (no GPU dependencies)
- Model size: ~25MB (well under 200MB limit)
- Memory efficient processing
- No network/internet access required

### Multilingual Support
- Unicode text handling for international documents
- Supports Japanese, Chinese, Arabic, and other non-Latin scripts
- Font-agnostic heading detection

## Architecture

The `PDFOutlineExtractor` class provides a clean API:

```python
from extractor.pdf_outline_extractor import PDFOutlineExtractor

extractor = PDFOutlineExtractor(max_pages=50, heading_levels=3)
outline = extractor.extract(pdf_path)
```

### Key Features
- Dual parser support (PyMuPDF primary, pdfminer fallback)
- Configurable heading depth and page limits
- Robust error handling and validation
- Extensible for future ML model integration

## Scoring Optimization

- **Heading Detection Accuracy**: Multi-heuristic approach for high precision/recall
- **Performance**: Optimized parsing and efficient memory usage
- **Multilingual Bonus**: Full Unicode support with PyMuPDF
