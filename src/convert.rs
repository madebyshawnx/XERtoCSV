//! Core XER -> CSV conversion logic.
//!
//! An `.xer` file is tab-delimited text made up of stacked tables. Each line
//! starts with a two-character marker:
//!   * `%T` - start of a new table (the rest of the line is the table name)
//!   * `%F` - the field/column names for the current table (tab-separated)
//!   * `%R` - a data record for the current table (tab-separated)
//! Everything else (`ERMHDR`, `%E`, blank lines) is ignored.
//!
//! For each `.xer` file we create a subdirectory named after the file. Inside it
//! we write one `<TableName>.csv` per table, plus a master `<FileName>.xlsx`
//! workbook that holds every table on its own sheet.

use csv::WriterBuilder;
use rust_xlsxwriter::Workbook;
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
    /// Number of master Excel workbooks written (one per file with tables).
    pub workbooks_written: usize,
}

/// One parsed table from an `.xer` file.
struct Table {
    name: String,
    headers: Vec<String>,
    rows: Vec<Vec<String>>,
}

/// Convert every `.xer` file found anywhere under `input_dir` under `output_dir`.
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
        if tables > 0 {
            summary.workbooks_written += 1;
        }
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

/// Parse one `.xer` file, write a CSV per table plus a master workbook, and
/// return how many tables were written.
fn process_file(file_path: &Path, output_dir: &Path) -> io::Result<usize> {
    let file = File::open(file_path)?;
    let mut reader = BufReader::new(file);

    let mut bytes = Vec::new();
    reader.read_to_end(&mut bytes)?;
    let content = decode_xer(&bytes);
    // Strip a leading byte-order mark so the first line's marker is recognized.
    let content = content.strip_prefix('\u{feff}').unwrap_or(&content);

    let tables = parse_tables(content);
    if tables.is_empty() {
        return Ok(0);
    }

    // Group output by the source file's base name, e.g. ProjectA.xer -> ProjectA/.
    let base_name = file_path
        .file_stem()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_else(|| "output".to_string());
    let output_subdir = output_dir.join(&base_name);
    fs::create_dir_all(&output_subdir)?;

    // One CSV per table.
    for table in &tables {
        save_csv(table, &output_subdir)?;
    }

    // One master workbook with every table on its own sheet.
    save_workbook(&tables, &output_subdir, &base_name)?;

    Ok(tables.len())
}

/// Split the file content into tables using the `%T`, `%F`, and `%R` markers.
fn parse_tables(content: &str) -> Vec<Table> {
    let mut tables: Vec<Table> = Vec::new();
    let mut current: Option<Table> = None;

    for line in content.lines() {
        let line = line.trim();

        if let Some(rest) = line.strip_prefix("%T") {
            if let Some(table) = current.take() {
                tables.push(table);
            }
            current = Some(Table {
                name: rest.trim().to_string(),
                headers: Vec::new(),
                rows: Vec::new(),
            });
        } else if let Some(rest) = line.strip_prefix("%F") {
            if let Some(table) = current.as_mut() {
                table.headers = rest.trim().split('\t').map(String::from).collect();
            }
        } else if let Some(rest) = line.strip_prefix("%R") {
            if let Some(table) = current.as_mut() {
                let mut row: Vec<String> = rest.trim().split('\t').map(String::from).collect();
                // Pad short rows so every row matches the header width.
                if row.len() < table.headers.len() {
                    row.extend(vec![String::new(); table.headers.len() - row.len()]);
                }
                table.rows.push(row);
            }
        }
    }

    if let Some(table) = current.take() {
        tables.push(table);
    }
    tables
}

fn save_csv(table: &Table, output_subdir: &Path) -> io::Result<()> {
    let csv_file_name = format!("{}.csv", table.name.replace(' ', "_"));
    let csv_file_path = output_subdir.join(csv_file_name);

    let file = File::create(csv_file_path)?;
    // `flexible` so a row that does not match the header width never aborts the file.
    let mut wtr = WriterBuilder::new().flexible(true).from_writer(file);

    wtr.write_record(&table.headers)?;
    for row in &table.rows {
        wtr.write_record(row)?;
    }
    wtr.flush()?;
    Ok(())
}

/// Write a single `.xlsx` workbook with one sheet per table.
fn save_workbook(tables: &[Table], output_subdir: &Path, base_name: &str) -> io::Result<()> {
    let mut workbook = Workbook::new();
    let mut used_sheet_names: Vec<String> = Vec::new();

    for table in tables {
        let sheet_name = unique_sheet_name(&table.name, &mut used_sheet_names);
        let worksheet = workbook.add_worksheet();
        worksheet.set_name(&sheet_name).map_err(to_io)?;

        for (col, header) in table.headers.iter().enumerate() {
            worksheet
                .write_string(0, col as u16, header)
                .map_err(to_io)?;
        }
        for (r, row) in table.rows.iter().enumerate() {
            for (col, value) in row.iter().enumerate() {
                worksheet
                    .write_string((r + 1) as u32, col as u16, value)
                    .map_err(to_io)?;
            }
        }
    }

    let xlsx_path = output_subdir.join(format!("{}.xlsx", base_name));
    workbook.save(&xlsx_path).map_err(to_io)?;
    Ok(())
}

/// Turn a table name into a valid, unique Excel sheet name. Excel sheet names
/// must be 1 to 31 characters, cannot contain `\ / ? * [ ] :`, and must be
/// unique within the workbook.
fn unique_sheet_name(raw: &str, used: &mut Vec<String>) -> String {
    let mut name: String = raw
        .chars()
        .map(|c| match c {
            '\\' | '/' | '?' | '*' | '[' | ']' | ':' => '_',
            _ => c,
        })
        .collect();
    if name.trim().is_empty() {
        name = "Sheet".to_string();
    }
    if name.chars().count() > 31 {
        name = name.chars().take(31).collect();
    }

    let mut candidate = name.clone();
    let mut n = 2;
    while used.iter().any(|u| u.eq_ignore_ascii_case(&candidate)) {
        let suffix = format!("_{n}");
        let keep = 31usize.saturating_sub(suffix.len());
        let base: String = name.chars().take(keep).collect();
        candidate = format!("{base}{suffix}");
        n += 1;
    }
    used.push(candidate.clone());
    candidate
}

/// Decode raw `.xer` bytes into text. Primavera P6 usually exports Windows-1252
/// (ANSI), but some files are UTF-8. Honor a byte-order mark if present,
/// otherwise use UTF-8 when the bytes are valid UTF-8 and fall back to
/// Windows-1252 so accented names and symbols are preserved, not corrupted.
fn decode_xer(bytes: &[u8]) -> String {
    if let Some((encoding, _)) = encoding_rs::Encoding::for_bom(bytes) {
        return encoding.decode(bytes).0.into_owned();
    }
    match std::str::from_utf8(bytes) {
        Ok(text) => text.to_string(),
        Err(_) => encoding_rs::WINDOWS_1252.decode(bytes).0.into_owned(),
    }
}

fn to_io<E: std::fmt::Display>(e: E) -> io::Error {
    io::Error::new(io::ErrorKind::Other, e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_multiple_tables_with_headers_and_rows() {
        let xer = "ERMHDR\t7.0\n%T\tTASK\n%F\tid\tname\n%R\t1\tDig\n%R\t2\tPour\n%T\tPROJECT\n%F\tpid\n%R\t100\n%E";
        let tables = parse_tables(xer);
        assert_eq!(tables.len(), 2);
        assert_eq!(tables[0].name, "TASK");
        assert_eq!(tables[0].headers, vec!["id", "name"]);
        assert_eq!(tables[0].rows.len(), 2);
        assert_eq!(tables[1].name, "PROJECT");
        assert_eq!(tables[1].rows[0], vec!["100"]);
    }

    #[test]
    fn pads_rows_that_are_shorter_than_the_header() {
        let xer = "%T\tT\n%F\ta\tb\tc\n%R\t1\n%E";
        let tables = parse_tables(xer);
        assert_eq!(tables[0].rows[0], vec!["1", "", ""]);
    }

    #[test]
    fn decodes_windows_1252_without_corruption() {
        // 0xA3 is the Pound Sterling sign in Windows-1252 and is invalid UTF-8.
        let bytes = b"%T\tCUR\n%F\tsym\n%R\t\xA3\n";
        let text = decode_xer(bytes);
        assert!(text.contains('\u{00A3}'), "pound sign should be preserved");
        assert!(!text.contains('\u{FFFD}'), "no replacement characters");
    }

    #[test]
    fn decodes_valid_utf8_as_utf8() {
        let bytes = "%R\tcafé\n".as_bytes();
        assert!(decode_xer(bytes).contains("café"));
    }

    #[test]
    fn strips_a_leading_utf8_bom() {
        let bytes = b"\xEF\xBB\xBF%T\tTASK\n%F\tid\n%R\t1\n";
        let text = decode_xer(bytes);
        let text = text.strip_prefix('\u{feff}').unwrap_or(&text);
        let tables = parse_tables(text);
        assert_eq!(tables.len(), 1);
        assert_eq!(tables[0].name, "TASK");
    }

    #[test]
    fn sheet_names_are_sanitized_truncated_and_unique() {
        let mut used = Vec::new();
        assert_eq!(unique_sheet_name("TASK", &mut used), "TASK");
        // Forbidden characters are replaced.
        assert_eq!(unique_sheet_name("A/B:C", &mut used), "A_B_C");
        // Duplicates get a numeric suffix.
        assert_eq!(unique_sheet_name("TASK", &mut used), "TASK_2");
        // Over-long names are clamped to 31 characters.
        let long = unique_sheet_name(&"X".repeat(50), &mut used);
        assert_eq!(long.chars().count(), 31);
    }
}
