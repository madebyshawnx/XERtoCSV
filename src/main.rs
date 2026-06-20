// Hide the console window on Windows release builds so double-clicking the .exe
// opens just the app window, not a black terminal behind it.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod convert;

use eframe::egui;
use std::path::{Path, PathBuf};

fn main() -> eframe::Result<()> {
    // Backward-compatible command-line mode: if the user passes
    // `<input_dir> <output_dir>`, run the old behavior and skip the window.
    let args: Vec<String> = std::env::args().collect();
    if args.len() == 3 {
        match convert::convert_dir(Path::new(&args[1]), Path::new(&args[2])) {
            Ok(summary) => {
                println!(
                    "Done. Processed {} file(s), wrote {} CSV table(s) and {} master Excel workbook(s).",
                    summary.files_processed.len(),
                    summary.tables_written,
                    summary.workbooks_written
                );
            }
            Err(e) => {
                eprintln!("Error: {e}");
                std::process::exit(1);
            }
        }
        return Ok(());
    }

    // Otherwise, launch the desktop app.
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([620.0, 480.0])
            .with_min_inner_size([460.0, 360.0]),
        ..Default::default()
    };

    eframe::run_native(
        "XER to CSV Converter",
        options,
        Box::new(|_cc| Ok(Box::<XerApp>::default())),
    )
}

/// Where the input is coming from.
enum InputSource {
    /// A folder that will be scanned recursively for `.xer` files.
    Folder(PathBuf),
    /// A specific set of `.xer` files the user picked.
    Files(Vec<PathBuf>),
}

#[derive(Default)]
struct XerApp {
    input: Option<InputSource>,
    output: Option<PathBuf>,
    status: String,
    last_run_ok: bool,
}

impl eframe::App for XerApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.add_space(8.0);
            ui.heading("XER to CSV Converter");
            ui.label("Convert Primavera P6 .xer files into CSV tables. No command line needed.");
            ui.add_space(12.0);

            // ---- Step 1: choose the input ----
            ui.group(|ui| {
                ui.label(egui::RichText::new("1. Choose what to convert").strong());
                ui.add_space(4.0);
                ui.horizontal(|ui| {
                    if ui.button("📂  Choose a folder…").clicked() {
                        if let Some(dir) = rfd::FileDialog::new().pick_folder() {
                            self.input = Some(InputSource::Folder(dir));
                        }
                    }
                    if ui.button("📄  Choose .xer file(s)…").clicked() {
                        if let Some(files) = rfd::FileDialog::new()
                            .add_filter("XER files", &["xer"])
                            .pick_files()
                        {
                            if !files.is_empty() {
                                self.input = Some(InputSource::Files(files));
                            }
                        }
                    }
                });
                ui.add_space(4.0);
                ui.label(match &self.input {
                    Some(InputSource::Folder(p)) => format!("Selected folder: {}", p.display()),
                    Some(InputSource::Files(f)) => format!("Selected {} file(s)", f.len()),
                    None => "Nothing selected yet.".to_string(),
                });
            });

            ui.add_space(10.0);

            // ---- Step 2: choose the output ----
            ui.group(|ui| {
                ui.label(egui::RichText::new("2. Choose where to save the CSVs").strong());
                ui.add_space(4.0);
                if ui.button("💾  Choose output folder…").clicked() {
                    if let Some(dir) = rfd::FileDialog::new().pick_folder() {
                        self.output = Some(dir);
                    }
                }
                ui.add_space(4.0);
                ui.label(match &self.output {
                    Some(p) => format!("Saving to: {}", p.display()),
                    None => "No output folder chosen yet.".to_string(),
                });
            });

            ui.add_space(14.0);

            // ---- Step 3: convert ----
            let ready = self.input.is_some() && self.output.is_some();
            ui.add_enabled_ui(ready, |ui| {
                if ui
                    .add(egui::Button::new(egui::RichText::new("Convert").size(18.0)))
                    .clicked()
                {
                    self.run_conversion();
                }
            });
            if !ready {
                ui.label(
                    egui::RichText::new("Pick an input and an output folder to enable Convert.")
                        .weak(),
                );
            }

            // ---- Status ----
            if !self.status.is_empty() {
                ui.add_space(12.0);
                ui.separator();
                ui.add_space(6.0);
                let color = if self.last_run_ok {
                    egui::Color32::from_rgb(60, 160, 60)
                } else {
                    egui::Color32::from_rgb(200, 60, 60)
                };
                ui.label(egui::RichText::new(&self.status).color(color));
            }
        });
    }
}

impl XerApp {
    fn run_conversion(&mut self) {
        let Some(output) = self.output.clone() else {
            return;
        };

        let result = match &self.input {
            Some(InputSource::Folder(dir)) => convert::convert_dir(dir, &output),
            Some(InputSource::Files(files)) => convert::convert_files(files, &output),
            None => return,
        };

        match result {
            Ok(summary) if summary.files_processed.is_empty() => {
                self.last_run_ok = false;
                self.status = "No .xer files were found in the chosen location.".to_string();
            }
            Ok(summary) => {
                self.last_run_ok = true;
                self.status = format!(
                    "✅ Success! Converted {} file(s) into {} CSV table(s), plus {} master Excel workbook(s).\nSaved in: {}",
                    summary.files_processed.len(),
                    summary.tables_written,
                    summary.workbooks_written,
                    output.display()
                );
            }
            Err(e) => {
                self.last_run_ok = false;
                self.status = format!("❌ Something went wrong: {e}");
            }
        }
    }
}
