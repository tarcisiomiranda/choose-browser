#!/bin/sh

set -eu

PROJECT_ROOT=$(cd -- "$(dirname "$0")/.." && pwd)
INSTALL_DIR="${CHOOSE_BROWSER_INSTALL_DIR:-${HOME}/.local/bin}"
BINARY_NAME=choose-browser

cd "$PROJECT_ROOT"

printf '=== Choose Browser - local installer ===\n\n'

printf '[1/4] Building release binary...\n'
if command -v mise >/dev/null 2>&1; then
	mise run build
else
	cargo build --release
fi

printf '[2/4] Installing binary to %s...\n' "$INSTALL_DIR"
mkdir -p "$INSTALL_DIR"
install -m 0755 "target/release/${BINARY_NAME}" "${INSTALL_DIR}/${BINARY_NAME}"

case ":${PATH}:" in
	*":${INSTALL_DIR}:"*) ;;
	*)
		printf '\nWARNING: %s is not in your PATH.\n' "$INSTALL_DIR"
		printf 'Add this to your ~/.bashrc or ~/.zshrc:\n'
		printf "  export PATH=\"%s/.local/bin:\$PATH\"\n\n" "$HOME"
		;;
esac

printf '[3/4] Registering as default browser...\n'
"${INSTALL_DIR}/${BINARY_NAME}" --install

printf '[4/4] Applying desktop-environment specific settings...\n'
de=$(printf '%s' "${XDG_CURRENT_DESKTOP:-unknown}" | tr '[:upper:]' '[:lower:]')
printf 'Detected desktop environment: %s\n' "${XDG_CURRENT_DESKTOP:-unknown}"

case "$de" in
	*gnome*)
		if command -v gsettings >/dev/null 2>&1; then
			gsettings set org.gnome.desktop.default-applications.web-browser "${INSTALL_DIR}/${BINARY_NAME}" 2>/dev/null || true
		fi
		;;
	*cinnamon*)
		if command -v gsettings >/dev/null 2>&1; then
			gsettings set org.cinnamon.desktop.default-applications.web-browser "${INSTALL_DIR}/${BINARY_NAME}" 2>/dev/null || true
		fi
		;;
	*kde* | *plasma*)
		kdeglobals="${HOME}/.config/kdeglobals"
		if [ -f "$kdeglobals" ]; then
			if grep -q '\[General\]' "$kdeglobals"; then
				sed -i '/^\[General\]/,/^\[/ s|^BrowserApplication=.*|BrowserApplication=choose-browser.desktop|' "$kdeglobals"
			else
				printf '\n[General]\nBrowserApplication=choose-browser.desktop\n' >>"$kdeglobals"
			fi
		else
			mkdir -p "$(dirname "$kdeglobals")"
			printf '[General]\nBrowserApplication=choose-browser.desktop\n' >"$kdeglobals"
		fi
		if command -v kwriteconfig5 >/dev/null 2>&1; then
			kwriteconfig5 --file kdeglobals --group General --key BrowserApplication "choose-browser.desktop"
		elif command -v kwriteconfig6 >/dev/null 2>&1; then
			kwriteconfig6 --file kdeglobals --group General --key BrowserApplication "choose-browser.desktop"
		fi
		;;
	*)
		printf 'No DE-specific settings applied (using XDG defaults).\n'
		;;
esac

printf '\nInstallation complete!\n'
printf 'Choose Browser is now your default browser.\n'
