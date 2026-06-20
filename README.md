# XER to CSV Converter

This is a simple desktop app that converts Primavera P6 `.xer` schedule files into CSV files. It creates one CSV file per table. There is no installation, no command line, and no technical setup required.

## Download and Run (Windows)

Click the link below to download the app. The file will download immediately.

[**Download XERtoCSV-Windows.exe**](https://raw.githubusercontent.com/madebyshawnx/XERtoCSV/main/download/XERtoCSV-Windows.exe)

After it downloads, follow these steps.

1. Double-click the downloaded file named `XERtoCSV-Windows.exe`.
2. In the app window, choose what you want to convert. You can pick a folder of `.xer` files or pick individual `.xer` files.
3. Choose the output folder where you want the CSV files to be saved.
4. Click the **Convert** button. The app will show a green message when it is done.

The app is fully self-contained, so there is nothing else to install.

### First-time warnings are normal

Because this app does not have a paid code-signing certificate, your computer may show a warning the first time you use it. These warnings are expected for any new app, and you can safely continue past them.

1. If your browser says the file is not commonly downloaded, choose **Keep** to allow it.
2. If Windows shows a blue box that says "Windows protected your PC," click **More info** and then click **Run anyway**.

## What You Get

For each `.xer` file, the app creates a subfolder named after that file. Inside that subfolder, it writes one `.csv` file for each table found in the original file.

```
your-output-folder/
  ProjectA/            (from ProjectA.xer)
    TASK.csv
    PROJWBS.csv
    CALENDAR.csv
  ProjectB/            (from ProjectB.xer)
    TASK.csv
    PROJWBS.csv
```

Each CSV file uses the table's column names as the first row, followed by one row for each record. Commas, quotes, and line breaks inside the data are escaped automatically so the CSV stays valid.

## Versions for Mac and Linux

The download above is the Windows version. Versions for Mac and Linux are built automatically when a new release is tagged. You can find them on the [Releases page](../../releases).

| System  | File                   |
| ------- | ---------------------- |
| Windows | `XERtoCSV-Windows.exe` |
| macOS   | `XERtoCSV-macOS`       |
| Linux   | `XERtoCSV-Linux`       |

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
