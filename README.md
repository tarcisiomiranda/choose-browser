# choose-browser

Pick which installed browser opens a link.

```sh
curl -fsSL https://raw.githubusercontent.com/tarcisiomiranda/choose-browser/main/install.sh | sh
```

Installs to `~/.local/bin` (or `/usr/local/bin` as root) and sets itself as the
default browser.

```sh
CHOOSE_BROWSER_VERSION=v0.1.0 sh          # pin a release
CHOOSE_BROWSER_SKIP_REGISTER=1 sh         # binary only
CHOOSE_BROWSER_INSTALL_DIR="$HOME/bin" sh # custom path
```

## Usage

```sh
choose-browser https://example.com
choose-browser --list
choose-browser --list-all
choose-browser --config-path
choose-browser --init-config
choose-browser --uninstall
```

`1`–`9` open a browser, `c` copies the URL, `Esc` closes.
Copy needs `wl-copy` or `xclip`.

## Config

`choose-browser --init-config` writes `~/.config/choose-browser/config.toml`.

```toml
[browsers]
show_only = []
hidden = []
order = []
```

IDs come from `--list-all`.

## Build

```sh
mise install
mise run install
```

## Uninstall

```sh
./uninstall.sh
```

## License

MIT
