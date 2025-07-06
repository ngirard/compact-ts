# compact-ts

A command-line utility for generating and converting compact, sortable timestamps.

The program produces timestamps in the format `YY-DOY-BASEMIN`, which is designed to be both human-readable and easily sortable. It can also expand these timestamps back into a standard format.

## Timestamp Format

The generated timestamp has the following structure: `YY-DOY-BASEMIN`

* **YY**: The two-digit year (e.g., `25` for 2025).
* **DOY**: The three-digit day of the year, zero-padded (e.g., `001`–`366`).
* **BASEMIN**: The minutes since midnight (a value from 0 to 1439), encoded in a numerical base and zero-padded to three characters.
    * **Default (Base-12):** Uses the characters `0-9` and `A-B`. This base is chosen for its efficient use of the 3-character space (~83.3%), where the maximum value `1439` is represented as `9BB`.
    * **Alternative (Base-36):** Uses the characters `0-9` and `A-Z`. Provided for applications requiring a larger character set, though it is less space-efficient (maximum value `1439` is `13Z` ; space utilization is only ~3%).

## Features

* Generates a compact timestamp for the current time.
* Converts standard timestamps to the compact format using the `generate --from` flag.
* Expands a compact timestamp back into a standard, readable format using the `expand` command.
* Parses a wide range of ISO 8601 and other common date/time formats for generation.
* Offers a configurable numerical base (`--base`) for both generation and expansion.
* Produces a fixed-width output ideal for sorting and use in filenames.
* Compiles to a single, self-contained binary with no runtime dependencies.

## Installation

### From pre-compiled binaries

Download the latest binary for your operating system from the [Releases](https://github.com/ngirard/compact-ts/releases) page.

### From source

If you have the Rust toolchain installed, you can install the latest version directly from the repository:
```sh
cargo install --git https://github.com/ngirard/compact-ts.git
```

## Usage

### Generating a compact timestamp

To generate a timestamp for the current time using the default Base-12:
```sh
$ compact-ts
25-181-956
```

To generate a timestamp using Base-36:
```sh
$ compact-ts --base b36
25-181-11U
```

### Converting from a standard timestamp

Use the `generate --from` flag to convert a given date/time string. The parser is flexible and accepts multiple formats. The examples below assume a UTC local timezone for consistent output.

```sh
# Full ISO 8601 with timezone
$ compact-ts generate --from "2025-06-30T22:42:05Z"
25-181-956

# ISO 8601 with offset, no seconds
$ compact-ts generate --from "2025-06-28T20:28+02:00"
25-179-784

# Date only (time defaults to 00:00)
$ compact-ts generate --from 2025-01-01
25-001-000

# Compact date and time to minute precision
$ compact-ts generate --from 20250630T2242
25-181-956

# Combining flags
$ compact-ts generate --from "2025-06-30T22:42" --base b36
25-181-11U
```

### Expanding a compact timestamp

Use the `expand` subcommand to convert a compact timestamp found within a string back to a standard format.

```sh
# Default expansion (assumes base-12)
$ compact-ts expand "backup-log-25-181-956.zip"
2025-06-30T22:42

# Specify the base if it's not the default
$ compact-ts expand "archive-25-181-11U.tar.gz" --base b36
2025-06-30T22:42

# Specify a custom output format
$ compact-ts expand "25-001-000" --format "%Y-%m-%d"
2025-01-01

$ compact-ts expand "25-001-000" --format "%A, %B %d, %Y at %I:%M %p"
Wednesday, January 01, 2025 at 12:00 AM
```

### Help

For a full list of options, use the `--help` flag. You can also get help for a specific subcommand.
```sh
$ compact-ts --help
A utility for compact, sortable timestamps.

Usage: compact-ts <COMMAND>

Commands:
  generate  Generate a compact timestamp (default behavior)
  expand    Expand a compact timestamp back to a standard format
  help      Print this message or the help of the given subcommand(s)

Options:
  -h, --help     Print help
  -V, --version  Print version

$ compact-ts expand --help
Expand a compact timestamp back to a standard format

Usage: compact-ts expand [OPTIONS] <INPUT_STRING>

Arguments:
  <INPUT_STRING>
          The string containing the compact timestamp to expand (e.g., "log-25-181-956.txt")

Options:
  -b, --base <BASE>
          The numerical base to assume for the minutes component of the timestamp
          [default: b12]
          [possible values: b12, b36]
  -f, --format <FORMAT>
          The output format for the expanded timestamp, using chrono specifiers.
          
          The format cannot include seconds or sub-second precision (e.g., %S, %s, %f)
          [default: %Y-%m-%dT%H:%M]
  -h, --help
          Print help
```

## Building from Source

### Prerequisites

* The Rust toolchain (install via [rustup.rs](https://rustup.rs/)).

### Steps

1. Clone the repository:
    ```sh
    git clone https://github.com/ngirard/compact-ts.git
    ```
2. Navigate to the project directory:
    ```sh
    cd compact-ts
    ```
3. Build the release binary:
    ```sh
    cargo build --release
    ```
    The executable will be located at `./target/release/compact-ts`.

4. Run tests:
    ```sh
    cargo test
    ```

## License

This project is licensed under the MIT License. See the `LICENSE` file for details.
