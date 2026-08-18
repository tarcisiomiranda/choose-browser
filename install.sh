#!/bin/sh

set -eu

REPOSITORY=${CHOOSE_BROWSER_REPOSITORY:-tarcisiomiranda/choose-browser}
VERSION=${CHOOSE_BROWSER_VERSION:-latest}
INSTALL_DIR=${CHOOSE_BROWSER_INSTALL_DIR:-}
SKIP_REGISTER=${CHOOSE_BROWSER_SKIP_REGISTER:-0}
BINARY_NAME=choose-browser

fail() {
	printf 'choose-browser installer: %s\n' "$*" >&2
	exit 1
}

command_exists() {
	command -v "$1" >/dev/null 2>&1
}

env_truthy() {
	value=$(printf '%s' "${1:-}" | tr '[:upper:]' '[:lower:]')
	case "$value" in
		1 | true | yes | on) return 0 ;;
		*) return 1 ;;
	esac
}

register_default_browser() {
	target_path=$1
	printf 'Registering as the default browser...\n'
	"$target_path" --install

	de=$(printf '%s' "${XDG_CURRENT_DESKTOP:-unknown}" | tr '[:upper:]' '[:lower:]')
	printf 'Detected desktop environment: %s\n' "${XDG_CURRENT_DESKTOP:-unknown}"

	case "$de" in
		*gnome*)
			if command_exists gsettings; then
				gsettings set org.gnome.desktop.default-applications.web-browser "$target_path" 2>/dev/null || true
			fi
			;;
		*cinnamon*)
			if command_exists gsettings; then
				gsettings set org.cinnamon.desktop.default-applications.web-browser "$target_path" 2>/dev/null || true
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
			if command_exists kwriteconfig5; then
				kwriteconfig5 --file kdeglobals --group General --key BrowserApplication "choose-browser.desktop"
			elif command_exists kwriteconfig6; then
				kwriteconfig6 --file kdeglobals --group General --key BrowserApplication "choose-browser.desktop"
			fi
			;;
		*)
			printf 'No DE-specific settings applied (using XDG defaults).\n'
			;;
	esac
}

command_exists curl || fail "curl is required"
command_exists install || fail "install is required"

case "$(uname -s)" in
	Linux) target_os=linux ;;
	*) fail "unsupported operating system: $(uname -s) (Linux only)" ;;
esac

case "$(uname -m)" in
	x86_64 | amd64) target_arch=amd64 ;;
	aarch64 | arm64) target_arch=arm64 ;;
	*) fail "unsupported architecture: $(uname -m)" ;;
esac

asset="${BINARY_NAME}-${target_os}-${target_arch}"
if [ "$VERSION" = latest ]; then
	download_url="https://github.com/${REPOSITORY}/releases/latest/download"
else
	download_url="https://github.com/${REPOSITORY}/releases/download/${VERSION}"
fi

temporary_directory=$(mktemp -d 2>/dev/null || mktemp -d -t choose-browser-install)
cleanup() {
	rm -rf "$temporary_directory"
}
trap cleanup EXIT HUP INT TERM

printf 'Downloading %s (%s/%s)...\n' "$BINARY_NAME" "$target_os" "$target_arch"
if ! curl --proto '=https' --tlsv1.2 -fsSL \
	"${download_url}/${asset}" \
	-o "${temporary_directory}/${asset}"; then
	fail "no release asset at ${download_url}/${asset}"
fi
if ! curl --proto '=https' --tlsv1.2 -fsSL \
	"${download_url}/checksums.txt" \
	-o "${temporary_directory}/checksums.txt"; then
	fail "no checksums.txt at ${download_url}/checksums.txt"
fi

expected_checksum=
while read -r checksum filename _; do
	filename=${filename#\*}
	if [ "$filename" = "$asset" ]; then
		expected_checksum=$checksum
		break
	fi
done <"${temporary_directory}/checksums.txt"

[ -n "$expected_checksum" ] || fail "checksum for ${asset} was not found"

if command_exists sha256sum; then
	checksum_output=$(sha256sum "${temporary_directory}/${asset}")
elif command_exists shasum; then
	checksum_output=$(shasum -a 256 "${temporary_directory}/${asset}")
else
	fail "sha256sum or shasum is required"
fi
actual_checksum=${checksum_output%% *}

if [ "$actual_checksum" != "$expected_checksum" ]; then
	fail "checksum mismatch for ${asset}"
fi
printf 'Checksum verified: %s\n' "$actual_checksum"

if [ -z "$INSTALL_DIR" ]; then
	if [ "$(id -u)" -eq 0 ] || [ -w /usr/local/bin ]; then
		INSTALL_DIR=/usr/local/bin
	else
		INSTALL_DIR=${HOME}/.local/bin
	fi
fi

target_path="${INSTALL_DIR}/${BINARY_NAME}"
if mkdir -p "$INSTALL_DIR" 2>/dev/null && [ -w "$INSTALL_DIR" ]; then
	install -m 0755 "${temporary_directory}/${asset}" "$target_path"
elif command_exists sudo; then
	printf 'Administrator permission is required to install in %s.\n' "$INSTALL_DIR"
	sudo mkdir -p "$INSTALL_DIR"
	sudo install -m 0755 "${temporary_directory}/${asset}" "$target_path"
else
	fail "cannot write to ${INSTALL_DIR}; set CHOOSE_BROWSER_INSTALL_DIR to a writable directory"
fi

if env_truthy "$SKIP_REGISTER"; then
	printf 'Skipped default-browser registration (CHOOSE_BROWSER_SKIP_REGISTER).\n'
else
	register_default_browser "$target_path"
fi

"$target_path" --list >/dev/null || fail "installed binary did not start correctly"

printf 'choose-browser installed successfully: %s\n' "$target_path"
case ":${PATH}:" in
	*":${INSTALL_DIR}:"*) ;;
	*) printf 'Add %s to PATH before running choose-browser.\n' "$INSTALL_DIR" ;;
esac
