# Nostalgia

Bring a bit of nostalgia to your day.

## 1. Introduction

Nostalgia is a command-line app for people who keep a personal collection of
wallpapers or terminal backgrounds. It reads a CSV library from your home
directory, applies optional category, subcategory, name, and type filters,
selects a matching image at random, and hands it to a configured desktop or
terminal adaptor. The project began as a Rust rewrite and extension of
[Pokemon-Terminal](https://github.com/LazoVelko/Pokemon-Terminal), with the
goal of supporting image collections beyond Pokémon. Nostalgia is open source
under GPL-3.0-or-later, so users can inspect, modify, and contribute to the
tool. It is intended for hobbyists who want a simple local collection and
adaptors rather than a separate image-management service.

## 2. Project description

### What it does

Keep image files and CSV metadata under `~/.nostalgia/`. Each database row
describes one image and can include category labels and foreground/background
colors. Nostalgia loads the CSV data, chooses a random row that matches the
filters in `config.toml`, builds the image path from the row, and runs each
enabled adaptor.

### Features and components

- **CSV database:** Loads CSV files recursively from `~/.nostalgia/` and
  supports filters for category, subcategory, name, and type.
- **Database validation:** `--validate-database` loads all rows and displays
  the dataframe schema, row count, per-category row counts, and
  the dataframe contents.
- **Adaptors:** Applies the selected image or its colors to supported
  desktop/terminal applications:

  | Adaptor | Behavior |
  | --- | --- |
  | `wallpaper` | Uses the platform wallpaper adaptor: macOS is supported; the Windows adaptor is currently a stub. |
  | `ghostty` | Updates Ghostty's image, opacity, and optional color settings. |
  | `warp` | Creates a JPEG-based Warp theme; select the generated theme in Warp's theme picker. |
  | `apple` | Changes Apple Terminal profile colors on macOS; Apple Terminal does not expose background images or opacity through its scripting dictionary. |
  | `darwin` | Sets the macOS desktop wallpaper. |

### Technical overview

Nostalgia is written in Rust (edition 2024). Clap defines the CLI, Confy and
Serde load configuration, Polars reads and queries CSV data, and tracing
provides structured application logging. The application is organized around
a database loader, user configuration, and an adaptor interface so the random
selection flow is separate from application-specific behavior.

## 3. Getting started

### Requirements and installation

Install a Rust toolchain with Cargo. Clone and build the repository:

```sh
git clone https://github.com/nostalgia-rs/nostalgia.git
cd nostalgia
cargo install --path .
```

Cargo downloads the Rust dependencies declared in `Cargo.toml`. Check the
available options with `nostalgia --help`.

On macOS, you can install Nostalgia with Homebrew:

```sh
brew tap nostalgia-rs/tap
brew install nostalgia
```

Or install directly without adding the tap first:

```sh
brew install nostalgia-rs/tap/nostalgia
```

### Configure an adaptor

Configuration is read from `~/.nostalgia/config.toml`. By default, only
`wallpaper` is enabled. For example:

```toml
enabled_adaptors = ["wallpaper", "ghostty"]
categories = ["ALL"]
subcategories = ["ALL"]
names = ["ALL"]
types = ["ALL"]
```

Use adaptor names such as `warp`, `ghostty`, and `apple` in
`enabled_adaptors`. On macOS, `wallpaper` selects the macOS desktop adaptor;
the Windows wallpaper adaptor is currently a stub.

### Add a database CSV and image

Nostalgia reads every CSV file below `~/.nostalgia/`. Save a CSV such as
`~/.nostalgia/database/images.csv` with this header and a row:

```csv
category,subcategory,index,name,dark_threshold,background_color,foreground_color,type,second_type
pokemon,fire,1,charmander,0.5,112233,aabbcc,Fire,
```

All columns are required in this order:

- `category` and `subcategory` provide directory names and values for their
  corresponding filters.
- `index` is the image filename stem and should be unique within its
  category/subcategory.
- `name` and `type` are values for the `names` and `types` filters.
- `background_color` and `foreground_color` are six-digit RGB hex colors,
  optionally prefixed with `#`; adaptors that support terminal colors use them.
- `dark_threshold` and `second_type` are required by the CSV schema but are
  currently unused.

Put the image at
`~/.nostalgia/<category>/<subcategory>/<index>.jpg`. For the example row,
create these directories and save the image here:

```sh
mkdir -p ~/.nostalgia/database ~/.nostalgia/pokemon/fire
```

```text
~/.nostalgia/pokemon/fire/1.jpg
```

Use JPEG files with the `.jpg` extension for all adaptors; Warp requires JPEG.
Filter values other than `ALL` must match the corresponding CSV values exactly.
`ALL` disables filtering for that field.

### Useful commands

```sh
nostalgia --get-config-location
nostalgia --download-assets
nostalgia --validate-database
nostalgia --print-config
nostalgia -v       # Info-level logs
nostalgia -vv      # Debug-level logs
```

`--validate-database` reads the full database without applying config filters
and exits with an error if the CSV data cannot be loaded.

`--download-assets` downloads the ZIP attached to the latest
[nostalgia-assets release](https://github.com/nostalgia-rs/nostalgia-assets/releases),
verifies its SHA-256 digest, and extracts it into `~/.nostalgia/`. The archive
is kept there. Each run checks GitHub for the latest release, even if a local
archive exists, so the command requires an internet connection. If the cached
archive's checksum matches the latest release, it is reused without downloading
it again; otherwise the latest ZIP is downloaded and verified before
extraction. Extraction overwrites files at paths present in the archive but
does not remove unrelated files or older versioned ZIPs.

## 4. Contributing

Bug reports, documentation improvements, and focused code changes are
welcome. Before opening a pull request:

1. Make a focused change and add or update tests for behavior it affects.
2. Format and check the project, then run its tests:

   ```sh
   cargo fmt
   cargo clippy
   cargo test
   ```

3. Describe the change and include relevant test results in your pull request.

Please use clear Rust code that follows `rustfmt` and the repository's Clippy
lint settings. Open an issue for larger proposals so the scope can be discussed
before implementation. Contributions are subject to the project's
[GPL-3.0-or-later license](LICENSE).

## 5. Project status

Nostalgia is under active development. macOS wallpaper, Ghostty configuration,
Warp themes, and Apple Terminal profile colors are implemented. The Windows
wallpaper adaptor remains a stub, and Apple Terminal does not support image
backgrounds or opacity through the interface used by Nostalgia. See the issue
tracker for current bugs and proposed work.

## 6. Resources

- **Repository:** [nostalgia-rs/nostalgia](https://github.com/nostalgia-rs/nostalgia)
- **Issues and feature requests:** [GitHub Issues](https://github.com/nostalgia-rs/nostalgia/issues)
- **License:** [GPL-3.0-or-later](LICENSE)
- **CLI reference:** Run `nostalgia --help` for the options available in your build.
