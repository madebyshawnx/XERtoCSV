//! Core XER -> CSV conversion logic.
//!
//! An `.xer` file is tab-delimited text made up of stacked tables. Each line
//! starts with a two-character marker:
//!   * `%T` - start of a new table (the rest of the line is the table name)
//!   * `%F` - the field/column names for the current table (tab-separated)
//!   * `%R` - a data record for the current table (tab-separated)
//! Everything else (`ERMHDR`, `%E`, blank lines) is ignored.
//!
//! For each table we write one `<TableName>.csv` file into a subdirectory named
//! after the source `.xer` file.

use csv::Writer;
use std::fs::{self, File};
use std::io::{self, BufReader, Read};
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

/// What happened during a conversion run, so the UI can report it.
#[derive(Default)]
pub struct ConversionSummary {
    /// Names of the `.xer` files that were processed.
    pub files_processed: Vec<String>,
    /// Total number of CSV tables written across all files.
    pub tables_written: usize,
}

/// Convert every `.xer` file found anywhere under `input_dir` into CSVs under
/// `output_dir`.
pub fn convert_dir(input_dir: &Path, output_dir: &Path) -> io::Result<ConversionSummary> {
    if !input_dir.is_dir() {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!("Input path is not a folder: {}", input_dir.display()),
        ));
    }

    let xer_files: Vec<PathBuf> = WalkDir::new(input_dir)
        .into_iter()
        .filter_map(Result::ok)
        .map(|e| e.path().to_path_buf())
        .filter(|p| p.is_file() && has_xer_extension(p))
        .collect();

    convert_files(&xer_files, output_dir)
}

/// Convert a specific list of `.xer` files into CSVs under `output_dir`.
pub fn convert_files(files: &[PathBuf], output_dir: &Path) -> io::Result<ConversionSummary> {
    fs::create_dir_all(output_dir)?;

    let mut summary = ConversionSummary::default();
    for path in files {
        let tables = process_file(path, output_dir)?;
        summary.tables_written += tables;
        summary
            .files_processed
            .push(path.file_name().unwrap_or_default().to_string_lossy().into_owned());
    }
    Ok(summary)
}

fn has_xer_extension(path: &Path) -> bool {
    path.extension()
        .and_then(|s| s.to_str())
        .map(|e| e.eq_ignore_ascii_case("xer"))
        .unwrap_or(false)
}

/// Parse one `.xer` file and write its tables as CSVs. Returns how many tables
/// were written.
fn process_file(file_path: &Path, output_dir: &Path) -> io::Result<usize> {
    let file = File::open(file_path)?;
    let mut reader = BufReader::new(file);

    // Read the whole file, decoding lossily: .xer files often contain bytes
    // that are not valid UTF-8.
    let mut content = Vec::new();
    reader.read_to_end(&mut content)?;
    let content = String::from_utf8_lossy(&content);
    // Strip a leading byte-order mark so the first line's marker is recognized.
    let content = content.strip_prefix('\u{feff}').unwrap_or(&content);

    // Group output by the source file's base name, e.g. ProjectA.xer -> ProjectA/.
    let base_name = file_path
        .file_stem()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_else(|| "output".to_string());
    let output_subdir = output_dir.join(base_name);
    fs::create_dir_all(&output_subdir)?;

    let mut tables_written = 0;
    let mut section_name = String::new();
    let mut headers: Vec<String> = Vec::new();
    let mut rows: Vec<Vec<String>> = Vec::new();

    for line in content.lines() {
        let line = line.trim();

        if let Some(rest) = line.strip_prefix("%T") {
            // A new table begins: flush the previous one first.
            if !section_name.is_empty() {
                save_csv(&section_name, &headers, &rows, &output_subdir)?;
                tables_written += 1;
            }
            section_name = rest.trim().to_string();
            headers.clear();
            rows.clear();
        } else if let Some(rest) = line.strip_prefix("%F") {
            headers = rest.trim().split('\t').map(String::from).collect();
        } else if let Some(rest) = line.strip_prefix("%R") {
            let mut row: Vec<String> = rest.trim().split('\t').map(String::from).collect();
            // Pad short rows so every row matches the header width.
            if row.len() < headers.len() {
                row.extend(vec![String::new(); headers.len() - row.len()]);
            }
            rows.push(row);
        }
    }

    // Flush the final table, which has no `%T` after it.
    if !section_name.is_empty() {
        save_csv(&section_name, &headers, &rows, &output_subdir)?;
        tables_written += 1;
    }

    Ok(tables_written)
}

fn save_csv(
    section_name: &str,
    headers: &[String],
    rows: &[Vec<String>],
    output_subdir: &Path,
) -> io::Result<()> {
    let csv_file_name = format!("{}.csv", section_name.replace(' ', "_"));
    let csv_file_path = output_subdir.join(csv_file_name);

    let file = File::create(csv_file_path)?;
    let mut wtr = Writer::from_writer(file);

    wtr.write_record(headers)?;
    for row in rows {
        wtr.write_record(row)?;
    }
    wtr.flush()?;
    Ok(())
}
