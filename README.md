# compact-ts

A command-line utility for generating and converting compact, sortable timestamps.

The program produces timestamps in the format `YY-DOY-BASEMIN`, which is designed to be both human-readable and easily sortable.

## Timestamp Format

The generated timestamp has the following structure: `YY-DOY-BASEMIN`

* **YY**: The two-digit year (e.g., `25` for 2025).
* **DOY**: The three-digit day of the year, zero-padded (e.g., `001`–`366`).
* **BASEMIN**: The minutes since midnight (a value from 0 to 1439), encoded in a numerical base and zero-padded to three characters.
    * **Default (Base-12):** Uses the characters `0-9` and `A-B`. This base is chosen for its efficient use of the 3-character space (~83.3%), where the maximum value `1439` is represented as `9BB`.
    * **Alternative (Base-36):** Uses the characters `0-9` and `A-Z`. Provided for applications requiring a larger character set, though it is less space-efficient (maximum value `1439` is `13Z` ; space utilization is only ~3%).

## Features

* Generates a compact timestamp for the current time.
* Converts standard timestamps to the compact format using the `--from` flag.
* Parses a wide range of ISO 8601 and other common date/time formats.
* Offers a configurable numerical base (`--base`) for the minutes component.
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

### Basic usage

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

Use the `--from` flag to convert a given date/time string. The parser is flexible and accepts multiple formats.

```sh
# Full ISO 8601 with timezone
$ compact-ts --from "2025-06-30T22:42:05Z"
25-181-956

# Date only (time defaults to 00:00)
$ compact-ts --from 2025-01-01
25-001-000

# Compact date and time to minute precision
$ compact-ts --from 20250630T2242
25-181-956

# Combining flags
$ compact-ts --from "2025-06-30T22:42" --base b36
25-181-11U
```

### Help

For a full list of options, use the `--help` flag.
```sh
$ compact-ts --help
A utility to print a compact timestamp: YY-DOY-BASEMIN

Usage: compact-ts [OPTIONS]

Options:
  -b, --base <BASE>  The numerical base to use for encoding the minutes since midnight [default: b12] [possible values: b12, b36]
      --from <FROM>  Convert a specific timestamp instead of using the current time.
                     
                     Tries to parse multiple formats, including:
                     - ISO 8601 with timezone: 2025-06-30T22:42:05Z
                     - ISO 8601 compact:      20250630T224205Z
                     - Date with hyphens:       2025-06-30 (time defaults to 00:00:00)
                     - Date compact:          20250630   (time defaults to 00:00:00)
  -h, --help         Print help
  -V, --version      Print version
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
