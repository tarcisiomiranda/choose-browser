# choose-browser

Linux desktop helper that intercepts `http`/`https` links and lets you pick
which installed browser opens them.

## Requirements

- Linux with XDG desktop files
- Rust 1.97.1 (via [mise](https://mise.jdx.dev/) or rustup)
- `xdg-mime` / `xdg-settings` (usually from `xdg-utils`)
- `wl-copy` (Wayland) or `xclip` (X11) to copy the URL from the chooser

## Install

```bash
mise install
mise run install
```

Without mise:

```bash
cargo build --release
./install.sh
```

This builds the release binary, copies it to `~/.local/bin/choose-browser`,
writes `choose-browser.desktop`, and registers it as the default web browser.

## Usage

```bash
choose-browser https://example.com
choose-browser --list
choose-browser --list-all
choose-browser --config-path
choose-browser --init-config
choose-browser --uninstall
```

Shortcuts in the chooser: `1`–`9` open a browser, `c` copies the URL, `Esc` closes.

## Config

User config lives at `~/.config/choose-browser/config.toml`. Create a starter
file with `choose-browser --init-config`.

```toml
[browsers]
show_only = []
hidden = []
order = []
```

Use desktop-file IDs or visible names. `choose-browser --list-all` prints what
the machine detected.

## Development

```bash
mise install
mise run check
mise run test
mise run lint
mise run fmt
mise run dev -- https://example.com
```

## Uninstall

```bash
mise run uninstall
```

or `./uninstall.sh`.

## License

MIT. See [LICENSE](LICENSE).
