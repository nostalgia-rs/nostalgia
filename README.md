# nostalgia

Bring a bit of nostalgia to your day

> [!NOTE]
> This is a partial fork of [Pokemon-Terminal](https://github.com/LazoVelko/Pokemon-Terminal) to rewrite in rust and expand support for non-pokemon themes.

DONE: Darwin wallpaper
DONE: Ghostty background image and color (updates the Ghostty config file)
DONE: Warp JPEG background themes and colors (select the generated theme once in Warp)
DONE: Apple Terminal active profile colors
NOTE: Apple Terminal does not expose background images or opacity through its scripting dictionary.

Configure adaptors in `~/.nostalgia/config.toml` with `enabled_adaptors`, for example:

```toml
enabled_adaptors = ["wallpaper", "warp", "ghostty", "apple"]
```

By default, only `wallpaper` is enabled; it resolves to the native Windows or macOS wallpaper adaptor. Terminal adaptors can be enabled by name: `warp`, `ghostty`, and `apple` (Apple Terminal, macOS only for now).

## Adding images and database rows

Nostalgia reads every CSV file below `~/.nostalgia/` and selects a row matching
the configured `categories`, `subcategories`, `names`, and `types`. Put your
CSV wherever convenient inside that directory, for example
`~/.nostalgia/database/images.csv`. It must have a header with these columns in
this order (all columns are required):

```csv
category,subcategory,index,name,dark_threshold,background_color,foreground_color,type,second_type
pokemon,fire,1,charmander,0.5,112233,aabbcc,Fire,
```

The columns mean:

- `category` and `subcategory`: image directory names and values matched by
  their corresponding config filters.
- `index`: image filename stem and a unique ID within the category and
  subcategory.
- `name` and `type`: values matched by the `names` and `types` config filters.
- `background_color` and `foreground_color`: six-digit RGB hex colors, with or
  without a leading `#`, used by adaptors that change terminal colors.
- `dark_threshold` and `second_type`: required for the CSV schema but currently
  unused by the app.

For each row, put its JPEG image in a matching category/subcategory folder.
The image filename is the row's `index` plus `.jpg`; the category,
subcategory, and index values must match the directory and filename exactly.
For example:

```sh
mkdir -p ~/.nostalgia/database ~/.nostalgia/pokemon/fire
```

Save the CSV example above as `~/.nostalgia/database/images.csv` and the
matching JPEG image at:

```text
~/.nostalgia/pokemon/fire/1.jpg
```

Use JPEG files for all adaptors; Warp requires JPEG, and `.jpg` is the expected
extension. The `ALL` value disables filtering for a config field; other filter
values must match the CSV value exactly. Keep each CSV and its corresponding
images under `~/.nostalgia/`, and use a unique `index` for each image within
its category/subcategory.

## Validating the database

Run the following command to read all database CSV files without applying the
configured filters and display the row count, schema, per-category row counts,
and dataframe:

```sh
nostalgia --validate-database
```

The command reports CSV read or schema errors and exits unsuccessfully if the
database cannot be loaded.
