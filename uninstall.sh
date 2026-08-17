#!/bin/bash
set -e

INSTALL_DIR="${HOME}/.local/bin"
BINARY_NAME="choose-browser"

echo "=== Choose Browser - Uninstaller ==="
echo ""

# Run the built-in uninstall first
if [ -f "${INSTALL_DIR}/${BINARY_NAME}" ]; then
    echo "[1/3] Removing browser registration..."
    "${INSTALL_DIR}/${BINARY_NAME}" --uninstall
else
    echo "[1/3] Binary not found, cleaning up manually..."
    # Manual cleanup of .desktop file
    DESKTOP_FILE="${HOME}/.local/share/applications/choose-browser.desktop"
    if [ -f "${DESKTOP_FILE}" ]; then
        rm "${DESKTOP_FILE}"
        echo "Removed ${DESKTOP_FILE}"
    fi
fi

# Revert DE-specific settings
echo "[2/3] Reverting desktop-environment settings..."

DE="${XDG_CURRENT_DESKTOP:-unknown}"

case "${DE,,}" in
    *gnome*)
        if command -v gsettings &> /dev/null; then
            gsettings reset org.gnome.desktop.default-applications.web-browser 2>/dev/null || true
        fi
        ;;
    *cinnamon*)
        if command -v gsettings &> /dev/null; then
            gsettings reset org.cinnamon.desktop.default-applications.web-browser 2>/dev/null || true
        fi
        ;;
    *kde*|*plasma*)
        if command -v kwriteconfig5 &> /dev/null; then
            kwriteconfig5 --file kdeglobals --group General --key BrowserApplication ""
        elif command -v kwriteconfig6 &> /dev/null; then
            kwriteconfig6 --file kdeglobals --group General --key BrowserApplication ""
        fi
        ;;
esac

# Remove binary
echo "[3/3] Removing binary..."
if [ -f "${INSTALL_DIR}/${BINARY_NAME}" ]; then
    rm "${INSTALL_DIR}/${BINARY_NAME}"
    echo "Removed ${INSTALL_DIR}/${BINARY_NAME}"
else
    echo "Binary not found at ${INSTALL_DIR}/${BINARY_NAME}"
fi

# Update desktop database
update-desktop-database "${HOME}/.local/share/applications" 2>/dev/null || true

echo ""
echo "Choose Browser has been uninstalled."
echo "Your system will use the default browser set by your desktop environment."
