use anyhow::{Context, Result};
use clap::Parser;
use mupdf::pdf::{PdfDocument, PdfWriteOptions};
use redactor::{RedactionService, RedactionTarget};
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

mod patterns;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// PDF files or directories to process
    #[arg(required = true)]
    paths: Vec<PathBuf>,

    /// Custom literal substrings to redact
    #[arg(short, long)]
    custom: Vec<String>,

    /// Custom regex patterns to redact
    #[arg(short, long)]
    regex: Vec<String>,

    /// Output folder (default: 'redacted/')
    #[arg(short, long, default_value = "redacted")]
    output: PathBuf,

    /// Skip default PII patterns (SSN, Email, etc.)
    #[arg(long)]
    no_default: bool,

    /// Verbose output
    #[arg(short, long)]
    verbose: bool,
}

fn main() -> Result<()> {
    let args = Args::parse();

    // Ensure output directory exists
    if !args.output.exists() {
        std::fs::create_dir_all(&args.output)
            .with_context(|| format!("Failed to create output directory: {:?}", args.output))?;
    }

    // Initialize Redactor service
    let service = RedactionService::with_secure_strategy();

    // Collect all targets
    let mut targets = Vec::new();

    // 1. Default PII patterns
    if !args.no_default {
        for pii in patterns::DEFAULT_PATTERNS {
            targets.push(RedactionTarget::Regex(pii.regex.to_string()));
            if args.verbose {
                println!("Loaded PII pattern: {}", pii.name);
            }
        }
    }

    // 2. Custom substrings
    for s in &args.custom {
        // Escape literal string to be used as a regex
        let escaped = patterns::escape_literal(s);
        targets.push(RedactionTarget::Regex(escaped));
    }

    // 3. Custom regexes
    for r in &args.regex {
        targets.push(RedactionTarget::Regex(r.to_string()));
    }

    if args.verbose {
        println!("Loaded {} redaction targets in total.", targets.len());
    }

    // Process all paths
    for input_path in &args.paths {
        if input_path.is_file() {
            process_file(input_path, &args.output, &service, &targets, args.verbose)?;
        } else if input_path.is_dir() {
            for entry in WalkDir::new(input_path).into_iter().filter_map(|e| e.ok()) {
                if entry.file_type().is_file() {
                    let path = entry.path();
                    if path.extension().and_then(|s| s.to_str()) == Some("pdf") {
                        process_file(path, &args.output, &service, &targets, args.verbose)?;
                    }
                }
            }
        }
    }

    println!("All redaction jobs completed.");
    Ok(())
}

fn process_file(
    input: &Path,
    output_dir: &Path,
    service: &RedactionService,
    targets: &[RedactionTarget],
    verbose: bool,
) -> Result<()> {
    let filename = input.file_name().context("Invalid file name")?;
    let mut output_path = output_dir.to_path_buf();
    output_path.push(filename);

    if verbose {
        println!("Redacting {:?} -> {:?}", input, output_path);
    } else {
        println!("Processing: {:?}", filename);
    }

    let staging = tempfile::tempdir_in(output_dir).context("Failed to stage redaction output")?;
    let intermediate = staging.path().join("redacted.pdf");
    let result = service
        .redact(input, &intermediate, targets)
        .with_context(|| format!("Failed to redact file: {:?}", input))?;

    let completed = if result.has_redactions() {
        // MuPDF replaces page streams when applying redactions. Its default save
        // can retain the original streams as unreferenced objects.
        let document = PdfDocument::open(intermediate.to_str().context("Invalid staging path")?)
            .context("Failed to reopen staged redaction")?;
        let cleaned = staging.path().join("cleaned.pdf");
        let mut options = PdfWriteOptions::default();
        options.set_garbage(true);
        document
            .save_with_options(cleaned.to_str().context("Invalid staging path")?, options)
            .context("Failed to discard unreferenced PDF objects")?;
        cleaned
    } else {
        intermediate
    };

    std::fs::rename(completed, &output_path)
        .with_context(|| format!("Failed to save output: {:?}", output_path))?;
    Ok(())
}
