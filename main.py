# main.py
"""Entry-point script for Docker container.

Scans /app/input for PDF files and writes outline JSON files to /app/output
with the same base filename.
"""
import os
from pathlib import Path
import time

from extractor.pdf_outline_extractor import extract_outline_to_json

# Allow overriding via environment variables for flexibility when running locally or in Docker
_input_env = Path(os.getenv("INPUT_DIR", "/app/input"))
_output_env = Path(os.getenv("OUTPUT_DIR", "/app/output"))

# If the default `/app/...` paths do not exist (e.g. running outside Docker),
# fall back to local `input` and `output` directories next to this script.
if not _input_env.exists():
    _input_env = Path(__file__).parent / "input"
if not _output_env.exists():
    _output_env = Path(__file__).parent / "output"

INPUT_DIR = _input_env
OUTPUT_DIR = _output_env


def process_all_pdfs() -> None:
    if not INPUT_DIR.exists():
        print(f"Input directory {INPUT_DIR} not found – creating it.")
        INPUT_DIR.mkdir(parents=True, exist_ok=True)
        print("Place PDF files in this directory and re-run the script.")
        return
    # Ensure output directory is empty for fresh run
    if OUTPUT_DIR.exists():
        for f in OUTPUT_DIR.iterdir():
            try:
                if f.is_file():
                    f.unlink()
                elif f.is_dir():
                    # Remove any sub-directories recursively
                    import shutil
                    shutil.rmtree(f)
            except Exception as e:
                print(f"Warning: could not remove {f}: {e}")
    else:
        OUTPUT_DIR.mkdir(parents=True, exist_ok=True)

    pdf_files = list(INPUT_DIR.glob("*.pdf"))
    if not pdf_files:
        print("No PDF files found in", INPUT_DIR)
        return

    start_time = time.perf_counter()
    for pdf_file in pdf_files:
        json_path = OUTPUT_DIR / (pdf_file.stem + ".json")
        print(f"Processing {pdf_file.name} -> {json_path.name}")
        extract_outline_to_json(pdf_file, json_path)

    elapsed = time.perf_counter() - start_time
    print(f"Completed in {elapsed:.2f}s")


if __name__ == "__main__":
    process_all_pdfs()
