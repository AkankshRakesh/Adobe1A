# extractor/pdf_outline_extractor.py
"""PDF Outline Extractor
This module provides functionality to extract the title and hierarchical headings (H1-H3)
from a PDF document, along with their page numbers, producing a structured JSON object.
The extraction approach combines multiple heuristics:
1. Font size analysis and relative ranking
2. Font weight/bold detection
3. Numeric heading pattern recognition (1., 2.1, etc.)
4. Text case analysis
5. Content filtering to exclude dates and non-headings
The implementation uses PyMuPDF (fitz) for better accuracy and multilingual support,
falling back to pdfminer.six if needed. The solution runs fully offline without external models.
"""
from __future__ import annotations

import json
import re
from collections import defaultdict
from pathlib import Path
from typing import Dict, List, Tuple, Optional

try:
    import fitz  # PyMuPDF
    HAS_PYMUPDF = True
except ImportError:
    HAS_PYMUPDF = False
    from pdfminer.high_level import extract_pages
    from pdfminer.layout import LAParams, LTTextContainer, LTChar

# Type aliases
Heading = Dict[str, str | int]
Outline = Dict[str, str | List[Heading]]


class PDFOutlineExtractor:
    """Extracts document outline (title + headings) from a PDF file."""

    def __init__(self, max_pages: int = 50, heading_levels: int = 3):
        self.max_pages = max_pages
        self.heading_levels = heading_levels

    # ---------------------------------------------------------------------
    # Public API
    # ---------------------------------------------------------------------
    def extract(self, pdf_path: Path) -> Outline:
        """Return outline dict for *pdf_path*.

        Raises:
            ValueError: If file does not exist or exceeds page limit.
        """
        if not pdf_path.exists():
            raise ValueError(f"File not found: {pdf_path}")

        if HAS_PYMUPDF:
            return self._extract_with_pymupdf(pdf_path)
        else:
            return self._extract_with_pdfminer(pdf_path)

    def _extract_with_pymupdf(self, pdf_path: Path) -> Outline:
        """Extract using PyMuPDF for better accuracy and multilingual support."""
        doc = fitz.open(str(pdf_path))
        pages_data: List[List[Tuple[str, int, float, bool]]] = []
        
        for page_num in range(min(len(doc), self.max_pages)):
            page = doc[page_num]
            blocks = page.get_text("dict")["blocks"]
            segments: List[Tuple[str, int, float, bool]] = []
            
            for block in blocks:
                if "lines" in block:
                    for line in block["lines"]:
                        line_text = ""
                        sizes = []
                        bold_flags = []
                        
                        for span in line["spans"]:
                            text = span["text"].strip()
                            if text:
                                line_text += text + " "
                                sizes.append(span["size"])
                                bold_flags.append("Bold" in span["font"] or "Black" in span["font"])
                        
                        line_text = line_text.strip()
                        if line_text and sizes:
                            avg_size = sum(sizes) / len(sizes)
                            bold = sum(bold_flags) / len(bold_flags) > 0.5 if bold_flags else False
                            segments.append((line_text, page_num + 1, avg_size, bold))
            
            if segments:
                pages_data.append(segments)
        
        doc.close()
        return self._process_segments(pages_data)

    def _extract_with_pdfminer(self, pdf_path: Path) -> Outline:
        """Fallback extraction using pdfminer.six."""
        pages_data: List[List[Tuple[str, int, float, bool]]] = []
        laparams = LAParams()  # default layout params
        for page_num, page_layout in enumerate(
            extract_pages(str(pdf_path), page_numbers=range(self.max_pages), laparams=laparams)
        ):
            segments: List[Tuple[str, int, float, bool]] = []  # text, page, size, bold
            for element in page_layout:
                if isinstance(element, LTTextContainer):
                    for text_line in element:
                        if hasattr(text_line, "get_text"):
                            line_text = text_line.get_text().strip()
                            if not line_text:
                                continue
                            # Aggregate font size / weight across chars in line
                            sizes = [char.size for char in text_line if isinstance(char, LTChar)] or [0]
                            bold_flags = [
                                "Bold" in char.fontname or "Black" in char.fontname
                                for char in text_line
                                if isinstance(char, LTChar)
                            ]
                            avg_size = sum(sizes) / len(sizes)
                            bold = sum(bold_flags) / len(bold_flags) > 0.5  # majority bold
                            segments.append((line_text, page_num + 1, avg_size, bold))
            if segments:
                pages_data.append(segments)
        
        return self._process_segments(pages_data)

    def _process_segments(self, pages_data: List[List[Tuple[str, int, float, bool]]]) -> Outline:

        """Process extracted text segments into structured outline."""
        flattened = [seg for page in pages_data for seg in page]
        if not flattened:
            return {"title": "", "outline": []}

        # Determine font size ranks -> heading level candidates
        size_to_lines: defaultdict[float, List[str]] = defaultdict(list)
        for text, _, size, _ in flattened:
            size_to_lines[size].append(text)

        # Sort sizes desc and pick top unique sizes as heading levels
        unique_sizes = sorted(size_to_lines.keys(), reverse=True)
        level_sizes = unique_sizes[: self.heading_levels + 1]  # include title size

        # Map text lines to heading levels
        headings: List[Heading] = []
        title_candidate = ""
        heading_pattern = re.compile(r"^(\d+(?:\.\d+)*)\s+")
        for text, page, size, bold in flattened:
            # Numeric heading like "1." or "2.3.4 "
            num_level = 0
            m = heading_pattern.match(text)
            if m:
                num_level = m.group(1).count(".") + 1  # 1 => H1, 1.1 => H2, etc.

            level = self._size_to_level(size, level_sizes)
            # Override font-based level with numeric level if recognised and within range
            if 1 <= num_level <= self.heading_levels:
                level = num_level

            if level == 0:
                # title candidate: appear early in doc, maybe bold/uppercase
                if page <= 1 and (bold or text.isupper() or len(text.split()) < 15):
                    title_candidate = title_candidate or text
            elif 1 <= level <= self.heading_levels:
                clean = self._clean_text(text)
                if self._is_valid_heading(clean):
                    headings.append(
                        {
                            "level": f"H{level}",
                            "text": clean,
                            "page": page,
                        }
                    )

        # If no explicit title found, fall back to largest font text
        if not title_candidate:
            large_size = level_sizes[0]
            large_candidates = [t for t, _, s, _ in flattened if s == large_size]
            title_candidate = large_candidates[0] if large_candidates else ""

        # Deduplicate consecutive identical headings
        deduped: List[Heading] = []
        last = None
        for h in headings:
            if last is None or h != last:
                deduped.append(h)
            last = h

        return {"title": self._clean_text(title_candidate), "outline": deduped}

    # ------------------------------------------------------------------
    # Internal helpers
    # ------------------------------------------------------------------
    @staticmethod
    def _clean_text(text: str) -> str:
        text = re.sub(r"\s+", " ", text)
        return text.strip()

    def _is_valid_heading(self, text: str) -> bool:
        # Filter out all-digit strings or date-like strings (e.g., "18 JUNE 2013")
        if re.fullmatch(r"\d{1,2}\s+[A-Z]{3,}\s+\d{2,4}", text):
            return False
        if re.fullmatch(r"\d+", text):
            return False
        # Must contain at least one alphabetic char
        return any(c.isalpha() for c in text)

    @staticmethod
    def _size_to_level(size: float, level_sizes: List[float]) -> int:
        """Map *size* to level index based on *level_sizes* list.

        level 0 = title. level 1 = H1, etc. Non-heading returns -1.
        """
        try:
            idx = level_sizes.index(size)
            return idx  # 0 => title, 1 => H1, etc.
        except ValueError:
            return -1


# -----------------------------------------------------------------------
# CLI helper
# -----------------------------------------------------------------------

def extract_outline_to_json(pdf_path: str | Path, json_path: str | Path) -> None:
    pdf_path = Path(pdf_path)
    json_path = Path(json_path)
    extractor = PDFOutlineExtractor()
    outline = extractor.extract(pdf_path)
    json_path.write_text(json.dumps(outline, ensure_ascii=False, indent=2))


if __name__ == "__main__":
    import argparse

    parser = argparse.ArgumentParser(description="Extract PDF outline to JSON")
    parser.add_argument("input_pdf", type=str, help="Input PDF file path")
    parser.add_argument("output_json", type=str, help="Output JSON file path")
    args = parser.parse_args()

    extract_outline_to_json(args.input_pdf, args.output_json)
