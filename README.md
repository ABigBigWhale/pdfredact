# PDF Redact

A Rust-based CLI tool that removes matching PDF page text and marks the redacted areas.

## Features
- Redact common PII using built-in regex:
  - Emails
  - Social Security Numbers (SSN)
  - Phone Numbers
  - Basic home addresses
  - Age mentions
  - Biometric keywords (fingerprint, retina, etc.)
- Support for custom literal substrings.
- Support for custom regular expressions.
- Recursive processing of directories.
- Outputs results to a dedicated folder (default: `redacted/`).

## Prerequisites

This tool uses the `redactor` crate which depends on **MuPDF**.

### macOS
```bash
brew install mupdf
```

### Linux (Ubuntu/Debian)
```bash
sudo apt-get install build-essential libx11-dev libxext-dev libxinerama-dev libxrandr-dev libxcursor-dev libglu1-mesa-dev mesa-common-dev libasound2-dev libfreetype6-dev libharfbuzz-dev libopenjp2-7-dev libjbig2dec0-dev
```

## Installation

```bash
cargo build --release
```

## Usage

### Basic Usage
Redact all PII from a single PDF:
```bash
./target/release/pdfredact sample.pdf
```

### Redact Custom Substrings
```bash
./target/release/pdfredact sample.pdf --custom "John Doe" --custom "123 Main St"
```

### Redact a Directory of PDFs
```bash
./target/release/pdfredact ./my_documents/ --output ./safe_docs/
```

### Using Custom Regex
```bash
./target/release/pdfredact sample.pdf --regex r"\b[A-Z]{3}-\d{5}\b"
```

### Options
```text
Usage: pdfredact [OPTIONS] <PATHS>...

Arguments:
  <PATHS>...  PDF files or directories to process

Options:
  -c, --custom <CUSTOM>  Custom literal substrings to redact
  -r, --regex <REGEX>    Custom regex patterns to redact
  -o, --output <OUTPUT>  Output folder (default: 'redacted/') [default: redacted]
      --no-default       Skip default PII patterns (SSN, Email, etc.)
  -v, --verbose          Verbose output
  -h, --help             Print help
  -V, --version          Print version
```

## How it works
The tool uses the `redactor` crate which leverages MuPDF's redaction API. It:
1.  Parses the PDF content.
2.  Identifies text matching the target patterns.
3.  Determines the exact coordinates of the text.
4.  **Physically removes** the text objects from the PDF structure.
5.  Draws a black rectangle over the redacted area.
6.  Discards unreferenced PDF objects, including replaced page streams, before publishing the result.

Redaction is limited to matching page text. Scanned images, form field values, metadata, annotations, and embedded attachments can still contain sensitive information. Files with no matches are copied as-is.
