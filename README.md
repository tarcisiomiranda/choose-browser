# choose-browser

Pick which installed browser opens a link.

Supports Linux and Windows 10/11 (x64).

## Install on Linux

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

## Install on Windows

Run in PowerShell:

```powershell
curl.exe -fsSL https://raw.githubusercontent.com/tarcisiomiranda/choose-browser/main/install.ps1 | Out-String | Invoke-Expression
```

Or use PowerShell's built-in downloader:

```powershell
irm https://raw.githubusercontent.com/tarcisiomiranda/choose-browser/main/install.ps1 | iex
```

Downloads the Windows executable, verifies its SHA-256 checksum, and installs to
`%LOCALAPPDATA%\Programs\ChooseBrowser` without administrator privileges. The
installer opens **Settings > Apps > Default apps**: select **Choose Browser**
for **HTTP** and **HTTPS** to use the chooser when opening links.

The release must include `choose-browser-windows-amd64.exe`. To build from a
checkout instead, follow the Windows build instructions below.

The same `CHOOSE_BROWSER_VERSION`, `CHOOSE_BROWSER_SKIP_REGISTER`, and
`CHOOSE_BROWSER_INSTALL_DIR` environment variables are supported. When running
a downloaded script, you can also pass `-Version`, `-InstallDir`, or `-SkipRegister`.

```powershell
& "$env:LOCALAPPDATA\Programs\ChooseBrowser\choose-browser.exe" --list-all
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
Copy uses the native clipboard on Windows. On Linux, it needs `wl-copy` or `xclip`.

On Windows, browsers are discovered from their registered installations,
including per-user and 32-bit installations. Portable browsers without Windows
registration are not detected. The chooser excludes itself from the list.

## Config

`choose-browser --init-config` writes `~/.config/choose-browser/config.toml`.
On Windows, it writes `%APPDATA%\choose-browser\config.toml`.

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

On Windows, install [Visual Studio Build Tools](https://visualstudio.microsoft.com/downloads/)
with **Desktop development with C++** and a Windows SDK, plus
[mise](https://mise.jdx.dev/installing-mise.html). Then run in PowerShell:

```powershell
mise trust
mise install rust python
mise run install
```

`mise run build` creates `target\release\choose-browser.exe`.
`mise run dev -- https://example.com` opens the chooser from source.
`mise run ci` runs the platform's checks, including offline installer tests.

## Uninstall

```sh
./uninstall.sh
```

On Windows, run `mise run uninstall` from the checkout or:

```powershell
.\uninstall.ps1
```

To remove only the browser registration, run the installed executable with
`--uninstall`. Configuration files are preserved. Select your preferred
browser in Windows Settings after uninstalling.

## License

MIT
