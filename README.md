# XER → CSV Converter

A simple desktop app that converts Primavera P6 `.xer` schedule files into CSV
files — one CSV per table. No installation, no command line, no technical setup.

---

## ⬇️ Download & Run (Windows)

**[Download XERtoCSV-Windows.exe](download/XERtoCSV-Windows.exe?raw=true)**

1. Click the link above, then click the **Download** button on the next page.
2. Double-click the downloaded `XERtoCSV-Windows.exe`.
3. Use the app:
   - **1. Choose what to convert** — pick a folder of `.xer` files, *or* pick
     individual `.xer` files.
   - **2. Choose where to save** — pick an output folder for the CSVs.
   - **3. Convert** — click it. Done.

That's it. The app is fully self-contained — nothing else to install.

> **Windows SmartScreen note:** because the app isn't code-signed, Windows may
> show a blue "Windows protected your PC" box the first time. Click
> **More info → Run anyway**. (This happens with any new, unsigned app.)

> **Mac / Linux:** versions for those systems are produced automatically on the
> [Releases page](../../releases). See *Other platforms* below.

---

## What you get

For each `.xer` file, the app creates a subfolder named after that file, and
inside it writes one `.csv` per table found in the file:

```
your-output-folder/
  ProjectA/            (from ProjectA.xer)
    TASK.csv
    PROJWBS.csv
    CALENDAR.csv
    ...
  ProjectB/            (from ProjectB.xer)
    ...
```

Each CSV has the table's column names as the first (header) row, followed by one
row per record. Commas, quotes, and newlines inside fields are escaped
automatically.

---

## Other platforms (Mac & Linux)

Tagged releases automatically build standalone apps for **Windows, macOS, and
Linux**. Grab the one for your system from the
[**Releases page**](../../releases):

| System  | File |
| ------- | ---- |
| Windows | `XERtoCSV-Windows.exe` |
| macOS   | `XERtoCSV-macOS` |
| Linux   | `XERtoCSV-Linux` |

---

## For developers

### Run from source

Requires [Rust](https://rustup.rs/).

```bash
cargo run --release
```

This opens the desktop app. You can also run it headless from the command line:

```bash
cargo run --release -- <input_directory> <output_directory>
```

### How it works

An `.xer` file is tab-delimited text made of stacked tables. Each line starts
with a marker:

| Marker | Meaning |
| ------ | ------- |
| `%T`   | start of a new table (the rest of the line is the table name) |
| `%F`   | the field/column names for that table |
| `%R`   | one data record |

Everything else (`ERMHDR`, `%E`, blanks) is ignored. The converter walks the
file line by line and writes each table to its own CSV. The conversion logic
lives in [`src/convert.rs`](src/convert.rs); the desktop window is in
[`src/main.rs`](src/main.rs).

### Building releases

Push a version tag to build and publish standalone apps for all three platforms:

```bash
git tag v1.0.0
git push origin v1.0.0
```

The [release workflow](.github/workflows/release.yml) compiles each platform and
attaches the binaries to a new GitHub Release.
