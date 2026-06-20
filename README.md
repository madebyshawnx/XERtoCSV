# XER to CSV Converter

This is a simple desktop app that converts Primavera P6 `.xer` schedule files into CSV files. For each `.xer` file it writes one CSV per table and a master Excel workbook that holds every table on its own tab. There is no installation, no command line, and no technical setup required.

## Download and Run (Windows)

Click the link below to download the app as a zip file. This link always points to the latest version.

[**Download XERtoCSV-Windows.zip**](https://github.com/madebyshawnx/XERtoCSV/releases/latest/download/XERtoCSV-Windows.zip)

After it downloads, follow these steps.

1. Right-click the downloaded `XERtoCSV-Windows.zip` and choose **Extract All**.
2. Open the extracted folder and double-click `XERtoCSV-Windows.exe`.
3. In the app window, choose what you want to convert. You can pick a folder of `.xer` files or pick individual `.xer` files.
4. Choose the output folder where you want the CSV files to be saved.
5. Click the **Convert** button. The app will show a green message when it is done.

The app is fully self-contained, so there is nothing else to install. The download is a zip file because web browsers block bare program files, and a zip downloads cleanly.

### First-time warning is normal

Because this app does not have a paid code-signing certificate, Windows may show a warning the first time you open it. This warning is expected for any new app, and you can safely continue past it. If Windows shows a blue box that says "Windows protected your PC," click **More info** and then click **Run anyway**.

You can always find the latest version on the [Releases page](https://github.com/madebyshawnx/XERtoCSV/releases/latest).

## What You Get

For each `.xer` file, the app creates a subfolder named after that file. Inside that subfolder, it writes one `.csv` file for each table, plus a master `.xlsx` Excel workbook named after the file that contains every table on its own tab.

```
your-output-folder/
  ProjectA/            (from ProjectA.xer)
    ProjectA.xlsx      (master workbook, one tab per table)
    TASK.csv
    PROJWBS.csv
    CALENDAR.csv
  ProjectB/            (from ProjectB.xer)
    ProjectB.xlsx
    TASK.csv
    PROJWBS.csv
```

Each CSV file uses the table's column names as the first row, followed by one row for each record. Commas, quotes, and line breaks inside the data are escaped automatically so the CSV stays valid.

The converter is built to handle any `.xer` file faithfully. It reads the Windows-1252 (ANSI) encoding that Primavera P6 normally exports, so accented names and symbols such as the Pound Sterling sign are preserved instead of being corrupted. It has been verified against real project files, reproducing every record with no data dropped.

## Versions for Mac and Linux

The download above is the Windows version. Versions for macOS and Linux are built automatically for each release and can be downloaded from the [Releases page](../../releases).

| System | File              |
| ------ | ----------------- |
| macOS  | `XERtoCSV-macOS`  |
| Linux  | `XERtoCSV-Linux`  |

## For Developers

### Run from source

This requires [Rust](https://rustup.rs/). Run the following command to open the desktop app.

```bash
cargo run --release
```

You can also run the converter from the command line without the window by passing an input folder and an output folder.

```bash
cargo run --release -- <input_directory> <output_directory>
```

### How it works

An `.xer` file is tab-delimited text made up of stacked tables. Each line begins with a short marker that tells the converter what the line contains.

| Marker | Meaning                                                  |
| ------ | -------------------------------------------------------- |
| `%T`   | The start of a new table. The rest of the line is its name. |
| `%F`   | The column names for the current table.                  |
| `%R`   | One data record for the current table.                   |

Every other line, such as the `ERMHDR` header and the `%E` end marker, is ignored. The converter reads the file line by line and writes each table to its own CSV file. The conversion logic lives in [src/convert.rs](src/convert.rs), and the desktop window is in [src/main.rs](src/main.rs).

### Build and publish releases

Push a version tag to build and publish standalone apps for all three platforms.

```bash
git tag v1.0.0
git push origin v1.0.0
```

The [release workflow](.github/workflows/release.yml) compiles each platform and attaches the finished apps to a new GitHub Release.
