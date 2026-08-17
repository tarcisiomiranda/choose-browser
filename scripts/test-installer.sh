#!/usr/bin/env bash

set -Eeuo pipefail

PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
TEMPORARY_DIRECTORY="$(mktemp -d)"

cleanup() {
	rm -rf "$TEMPORARY_DIRECTORY"
}
trap cleanup EXIT

case "$(uname -s)" in
	Linux) target_os=linux ;;
	*)
		printf 'Unsupported test operating system\n' >&2
		exit 1
		;;
esac

case "$(uname -m)" in
	x86_64 | amd64) target_arch=amd64 ;;
	aarch64 | arm64) target_arch=arm64 ;;
	*)
		printf 'Unsupported test architecture\n' >&2
		exit 1
		;;
esac

asset="choose-browser-${target_os}-${target_arch}"
release_directory="${TEMPORARY_DIRECTORY}/release"
mock_directory="${TEMPORARY_DIRECTORY}/mock-bin"
install_directory="${TEMPORARY_DIRECTORY}/install"
fake_home="${TEMPORARY_DIRECTORY}/home"
mkdir -p "$release_directory" "$mock_directory" "$fake_home"

cat >"${release_directory}/${asset}" <<'FAKE_BINARY'
#!/bin/sh
exit 0
FAKE_BINARY
chmod 0755 "${release_directory}/${asset}"

if command -v sha256sum >/dev/null 2>&1; then
	(
		cd "$release_directory"
		sha256sum "$asset" >checksums.txt
	)
else
	checksum=$(shasum -a 256 "${release_directory}/${asset}")
	printf '%s  %s\n' "${checksum%% *}" "$asset" >"${release_directory}/checksums.txt"
fi

cat >"${mock_directory}/curl" <<'MOCK_CURL'
#!/bin/sh

url=
output=
while [ "$#" -gt 0 ]; do
	case "$1" in
		-o)
			output=$2
			shift 2
			;;
		https://*)
			url=$1
			shift
			;;
		*) shift ;;
	esac
done

[ -n "$url" ] && [ -n "$output" ]
cp "${CHOOSE_BROWSER_TEST_RELEASE_DIRECTORY}/${url##*/}" "$output"
MOCK_CURL
chmod 0755 "${mock_directory}/curl"

CHOOSE_BROWSER_TEST_RELEASE_DIRECTORY="$release_directory" \
	CHOOSE_BROWSER_INSTALL_DIR="$install_directory" \
	CHOOSE_BROWSER_VERSION=v0.0.0 \
	CHOOSE_BROWSER_SKIP_REGISTER=1 \
	HOME="$fake_home" \
	PATH="${mock_directory}:${PATH}" \
	/bin/sh "${PROJECT_ROOT}/install.sh"

installed_binary="${install_directory}/choose-browser"
if [ ! -x "$installed_binary" ]; then
	printf 'Installer did not create an executable binary\n' >&2
	exit 1
fi
"$installed_binary" --list

# Registration path: fake binary still exits 0 on --install
install_directory_reg="${TEMPORARY_DIRECTORY}/install-reg"
mkdir -p "$install_directory_reg"
CHOOSE_BROWSER_TEST_RELEASE_DIRECTORY="$release_directory" \
	CHOOSE_BROWSER_INSTALL_DIR="$install_directory_reg" \
	CHOOSE_BROWSER_VERSION=v0.0.0 \
	HOME="$fake_home" \
	PATH="${mock_directory}:${PATH}" \
	/bin/sh "${PROJECT_ROOT}/install.sh"

if [ ! -x "${install_directory_reg}/choose-browser" ]; then
	printf 'Installer with registration did not create a binary\n' >&2
	exit 1
fi

printf 'Installer integration test passed\n'
