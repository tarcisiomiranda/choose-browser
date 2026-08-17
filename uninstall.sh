#!/bin/sh

set -eu

INSTALL_DIR="${CHOOSE_BROWSER_INSTALL_DIR:-${HOME}/.local/bin}"
BINARY_NAME=choose-browser

printf '=== Choose Browser - Uninstaller ===\n\n'

if [ -f "${INSTALL_DIR}/${BINARY_NAME}" ]; then
	printf '[1/3] Removing browser registration...\n'
	"${INSTALL_DIR}/${BINARY_NAME}" --uninstall
else
	printf '[1/3] Binary not found, cleaning up manually...\n'
	desktop_file="${HOME}/.local/share/applications/choose-browser.desktop"
	if [ -f "$desktop_file" ]; then
		rm "$desktop_file"
		printf 'Removed %s\n' "$desktop_file"
	fi
fi

printf '[2/3] Reverting desktop-environment settings...\n'
de=$(printf '%s' "${XDG_CURRENT_DESKTOP:-unknown}" | tr '[:upper:]' '[:lower:]')

case "$de" in
	*gnome*)
		if command -v gsettings >/dev/null 2>&1; then
			gsettings reset org.gnome.desktop.default-applications.web-browser 2>/dev/null || true
		fi
		;;
	*cinnamon*)
		if command -v gsettings >/dev/null 2>&1; then
			gsettings reset org.cinnamon.desktop.default-applications.web-browser 2>/dev/null || true
		fi
		;;
	*kde* | *plasma*)
		if command -v kwriteconfig5 >/dev/null 2>&1; then
			kwriteconfig5 --file kdeglobals --group General --key BrowserApplication ""
		elif command -v kwriteconfig6 >/dev/null 2>&1; then
			kwriteconfig6 --file kdeglobals --group General --key BrowserApplication ""
		fi
		;;
esac

printf '[3/3] Removing binary...\n'
if [ -f "${INSTALL_DIR}/${BINARY_NAME}" ]; then
	rm "${INSTALL_DIR}/${BINARY_NAME}"
	printf 'Removed %s\n' "${INSTALL_DIR}/${BINARY_NAME}"
else
	printf 'Binary not found at %s\n' "${INSTALL_DIR}/${BINARY_NAME}"
fi

update-desktop-database "${HOME}/.local/share/applications" 2>/dev/null || true

printf '\nChoose Browser has been uninstalled.\n'
printf 'Your system will use the default browser set by your desktop environment.\n'
