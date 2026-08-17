#!/bin/bash
set -e

INSTALL_DIR="${HOME}/.local/bin"
BINARY_NAME="choose-browser"

echo "=== Choose Browser - Installer ==="
echo ""

# Build release binary
echo "[1/4] Building release binary..."
if command -v mise >/dev/null 2>&1; then
    mise run build
else
    cargo build --release
fi

# Install binary
echo "[2/4] Installing binary to ${INSTALL_DIR}..."
mkdir -p "${INSTALL_DIR}"
cp "target/release/${BINARY_NAME}" "${INSTALL_DIR}/${BINARY_NAME}"
chmod +x "${INSTALL_DIR}/${BINARY_NAME}"

# Ensure ~/.local/bin is in PATH
if ! echo "$PATH" | grep -q "${INSTALL_DIR}"; then
    echo ""
    echo "WARNING: ${INSTALL_DIR} is not in your PATH."
    echo "Add this to your ~/.bashrc or ~/.zshrc:"
    echo "  export PATH=\"\${HOME}/.local/bin:\${PATH}\""
    echo ""
fi

# Register as default browser
echo "[3/4] Registering as default browser..."
"${INSTALL_DIR}/${BINARY_NAME}" --install

# Detect desktop environment and apply specific settings
echo "[4/4] Applying desktop-environment specific settings..."

DE="${XDG_CURRENT_DESKTOP:-unknown}"
echo "Detected desktop environment: ${DE}"

case "${DE,,}" in
    *gnome*)
        echo "Applying GNOME settings..."
        # GNOME uses xdg-settings which --install already handles
        # Also set via gsettings for GNOME-specific apps
        if command -v gsettings &> /dev/null; then
            gsettings set org.gnome.desktop.default-applications.web-browser "${INSTALL_DIR}/${BINARY_NAME}" 2>/dev/null || true
        fi
        ;;
    *cinnamon*)
        echo "Applying Cinnamon settings..."
        # Cinnamon also uses xdg-settings, but has its own preferred-apps
        if command -v gsettings &> /dev/null; then
            gsettings set org.cinnamon.desktop.default-applications.web-browser "${INSTALL_DIR}/${BINARY_NAME}" 2>/dev/null || true
        fi
        ;;
    *kde*|*plasma*)
        echo "Applying KDE/Plasma settings..."
        # KDE uses kdeglobals
        KDEGLOBALS="${HOME}/.config/kdeglobals"
        if [ -f "${KDEGLOBALS}" ]; then
            if grep -q "\[General\]" "${KDEGLOBALS}"; then
                sed -i '/^\[General\]/,/^\[/ s|^BrowserApplication=.*|BrowserApplication=choose-browser.desktop|' "${KDEGLOBALS}"
            else
                echo -e "\n[General]\nBrowserApplication=choose-browser.desktop" >> "${KDEGLOBALS}"
            fi
        else
            mkdir -p "$(dirname "${KDEGLOBALS}")"
            echo -e "[General]\nBrowserApplication=choose-browser.desktop" > "${KDEGLOBALS}"
        fi
        # Also set via kwriteconfig if available
        if command -v kwriteconfig5 &> /dev/null; then
            kwriteconfig5 --file kdeglobals --group General --key BrowserApplication "choose-browser.desktop"
        elif command -v kwriteconfig6 &> /dev/null; then
            kwriteconfig6 --file kdeglobals --group General --key BrowserApplication "choose-browser.desktop"
        fi
        ;;
    *)
        echo "No DE-specific settings applied (using XDG defaults)."
        ;;
esac

echo ""
echo "Installation complete!"
echo "Choose Browser is now your default browser."
echo ""
echo "Usage:"
echo "  choose-browser <url>      Open the browser chooser for a URL"
echo "  choose-browser --list     List detected browsers"
echo "  choose-browser --list-all List all detected browsers before config"
echo "  choose-browser --config-path  Show the config file path"
echo "  choose-browser --init-config  Create the default config file"
echo "  choose-browser --uninstall  Remove registration"
echo ""
echo "To fully uninstall, run: ${BINARY_NAME} --uninstall && rm ${INSTALL_DIR}/${BINARY_NAME}"
