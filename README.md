# choose-browser

Linux desktop helper that intercepts `http`/`https` links and lets you pick
which installed browser opens them.

Repository: [`git@github.com:tarcisiomiranda/choose-browser.git`](https://github.com/tarcisiomiranda/choose-browser)

## Install

```sh
curl -fsSL https://raw.githubusercontent.com/tarcisiomiranda/choose-browser/main/install.sh | sh
```

The installer detects `linux/amd64` or `linux/arm64`, downloads the latest
GitHub Release, verifies the SHA-256 checksum, installs the binary, and
registers choose-browser as the default web browser.

Pin a version or install directory:

```sh
curl -fsSL https://raw.githubusercontent.com/tarcisiomiranda/choose-browser/main/install.sh \
  | CHOOSE_BROWSER_VERSION=v0.1.0 CHOOSE_BROWSER_INSTALL_DIR="$HOME/.local/bin" sh
```

Binary only (skip default-browser registration):

```sh
curl -fsSL https://raw.githubusercontent.com/tarcisiomiranda/choose-browser/main/install.sh \
  | CHOOSE_BROWSER_SKIP_REGISTER=1 sh
```

As root, the binary goes to `/usr/local/bin`. Otherwise it uses `~/.local/bin`
when `/usr/local/bin` is not writable.

### From source

```sh
git clone git@github.com:tarcisiomiranda/choose-browser.git
cd choose-browser
mise install
mise run install
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

Shortcuts in the chooser: `1`–`9` open a browser, `c` copies the URL, `Esc` closes.

Copying the URL needs `wl-copy` (Wayland) or `xclip` (X11).

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

```sh
mise install
mise run check
mise run test
mise run lint
mise run fmt
mise run ci
mise run dev -- https://example.com
```

## Releases

Push a SemVer tag (or run the **Release** workflow with that tag):

```sh
git tag v0.1.0
git push origin v0.1.0
```

CI builds `choose-browser-linux-amd64` and `choose-browser-linux-arm64`, writes
`checksums.txt`, and publishes a GitHub Release. Optional notes:
`releases/<tag>.yaml` (see `releases/README.md`).

## Uninstall

```sh
mise run uninstall
```

or `./uninstall.sh`.

## License

MIT. See [LICENSE](LICENSE).
