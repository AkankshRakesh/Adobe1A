# PDF Outline Extractor

This project extracts a structured outline (Title, H1, H2, H3) from a PDF document and outputs it as a JSON file. It is designed to be a lightweight, offline, and CPU-only solution.

## Approach

The solution uses a hybrid, multi-pass approach to identify document structure without relying on a single feature like font size, which can be unreliable across different PDF generators.

1.  **Text and Font-Size Extraction**: The primary engine is the `lopdf` crate, which allows for low-level inspection of PDF content streams. A custom utility (`font_utils.rs`) iterates through each page's drawing operations to extract text runs along with their precise font sizes. This provides the foundational data for all subsequent analysis.

2.  **Font-Based Filtering**: To reduce the search space and eliminate noise from body text, footers, and headers, the program first determines a `heading_threshold`. This is calculated by finding the second-largest font size used in the document. Only text rendered at or above this size is considered a candidate for being a heading.

3.  **Heuristic-Based Heading Analysis**: Candidate lines that pass the font-size filter are then analyzed with a set of heuristics and regular expressions in `functions.rs`:
    *   **Numbered Headings**: A robust regex (`NUMBERED_HEADING`) matches various enumeration styles (e.g., `1.2.3`, `A.`, `IV.`). The nesting level (H1, H2, H3) is determined by the structure of the prefix.
    *   **Structural Headings**: Patterns for `Chapter X` or `Appendix A` are identified as high-level headings.
    *   **Stylistic Headings**: All-caps lines or title-cased phrases that are isolated by whitespace are scored and considered potential headings.

4.  **Title Extraction**: The document title is identified by scoring the first ~20 lines of the PDF. The scoring system rewards centrality, title case, common title keywords (e.g., "Report", "Guide"), and penalizes sentence-like structure or web links.

5.  **Hierarchy Establishment**: Finally, the collected headings are sorted by page number and de-duplicated to produce a clean, hierarchical outline.

This approach balances precision and recall while adhering to the strict performance and resource constraints (offline, ≤200MB, CPU-only).

## Libraries Used

*   **`lopdf`**: For low-level PDF parsing and content stream extraction.
*   **`pdf-extract`**: Used as a secondary, simpler text extraction method.
*   **`serde`**: For serializing the final outline structure into JSON.
*   **`clap`**: For parsing command-line arguments (`--input`, `--output`).
*   **`regex`**: Powers the pattern-matching engine for heading detection.
*   **`once_cell`**: For lazily initializing global regex patterns for efficiency.
*   **`anyhow`**: For streamlined error handling.

## How to Build and Run

You can run the solution using either Docker (recommended for submission) or a local Rust toolchain.

### Docker (for Submission)

This method uses the provided `Dockerfile` and is the expected way to run the solution for evaluation.

1.  **Build the Docker Image**:

    ```sh
    docker build --platform linux/amd64 -t outline-extractor:latest .
    ```

2.  **Run the Container**:

    Place your input PDFs in a directory (e.g., `input/`). The container will automatically process them and place the JSON results in an `output/` directory.

    ```sh
    docker run --rm -v $(pwd)/input:/app/input -v $(pwd)/output:/app/output --network none outline-extractor:latest
    ```

### Local Development (with Cargo)

This is useful for testing and development if you have the Rust toolchain installed.

1.  **Build the Project**:

    ```sh
    cargo build --release
    ```

2.  **Run the Executable**:

    Provide the path to a single input PDF and the desired output JSON file.

    ```sh
    ./target/release/adobe1a --input ./pdfs/sample.pdf --output ./output/sample.json
    ```
