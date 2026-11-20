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
