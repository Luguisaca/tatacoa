#!/usr/bin/env bash
set -euo pipefail

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$root"
[[ -z "$(git status --porcelain)" ]] || { echo 'El árbol debe estar limpio.' >&2; exit 1; }
command -v dpkg-deb >/dev/null || { echo 'Falta dpkg-deb.' >&2; exit 1; }
[[ "$(uname -m)" == x86_64 ]] || { echo 'Se requiere build Linux x86_64 nativo.' >&2; exit 1; }

head="$(git rev-parse --short=7 HEAD)"
epoch="$(git log -1 --format=%ct)"
out="${1:?Indique un directorio de salida nuevo}"
[[ ! -e "$out" ]] || { echo 'El destino ya existe.' >&2; exit 1; }
mkdir -p "$out"
out="$(realpath "$out")"
stage_root="$(mktemp -d "${TMPDIR:-/tmp}/tatacoa-package.XXXXXX")"
trap 'rm -rf -- "$stage_root"' EXIT
export SOURCE_DATE_EPOCH="$epoch"
export CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-$HOME/.cache/tatacoa-sp3-target}"
cargo build --locked --offline --release --workspace

base="tatacoa-0.1.0-alpha.1-$head-linux-x64"
appdir="$stage_root/$base.AppDir"
mkdir -p "$appdir/usr/bin" "$appdir/usr/share/applications"
install -m 0755 "$CARGO_TARGET_DIR/release/tatacoa-desktop" "$appdir/usr/bin/tatacoa-desktop"
install -m 0644 crates/tatacoa-desktop/icons/icon.png "$appdir/tatacoa.png"
ln -s tatacoa.png "$appdir/.DirIcon"
cat > "$appdir/AppRun" <<'EOF'
#!/bin/sh
HERE="$(dirname "$(readlink -f "$0")")"
exec "$HERE/usr/bin/tatacoa-desktop" "$@"
EOF
chmod 0755 "$appdir/AppRun"
cat > "$appdir/tatacoa.desktop" <<'EOF'
[Desktop Entry]
Type=Application
Name=TATACOA
Exec=tatacoa-desktop
Terminal=false
Categories=Utility;
Icon=tatacoa
EOF
cp "$appdir/tatacoa.desktop" "$appdir/usr/share/applications/tatacoa.desktop"
appimagetool_bin="${APPIMAGETOOL:-appimagetool}"
if command -v "$appimagetool_bin" >/dev/null; then
    [[ -n "${APPIMAGE_RUNTIME_FILE:-}" && -f "$APPIMAGE_RUNTIME_FILE" ]] || {
        echo 'Se requiere APPIMAGE_RUNTIME_FILE local para evitar descarga implícita.' >&2
        exit 1
    }
    APPIMAGE_EXTRACT_AND_RUN=1 ARCH=x86_64 "$appimagetool_bin" --runtime-file "$APPIMAGE_RUNTIME_FILE" "$appdir" "$out/$base-desktop.AppImage"
else
    echo 'AppImage NOT BUILT: appimagetool no está instalado en el host de build.' >&2
fi

deb="$out/$base-desktop.deb"
debroot="$stage_root/debroot"
mkdir -p "$debroot/DEBIAN" "$debroot/usr/bin" "$debroot/usr/share/applications" "$debroot/usr/share/doc/tatacoa" "$debroot/usr/share/icons/hicolor/64x64/apps"
install -m 0755 "$CARGO_TARGET_DIR/release/tatacoa-desktop" "$debroot/usr/bin/tatacoa-desktop"
install -m 0644 "$appdir/tatacoa.desktop" "$debroot/usr/share/applications/tatacoa.desktop"
install -m 0644 crates/tatacoa-desktop/icons/icon.png "$debroot/usr/share/icons/hicolor/64x64/apps/tatacoa.png"
install -m 0644 LICENSE "$debroot/usr/share/doc/tatacoa/LICENSE"
install -m 0644 NOTICE "$debroot/usr/share/doc/tatacoa/NOTICE"
cat > "$debroot/DEBIAN/control" <<'EOF'
Package: tatacoa-desktop
Version: 0.1.0~alpha.1
Section: utils
Priority: optional
Architecture: amd64
Maintainer: LUGUISACA <support@luguisaca.com>
Depends: libwebkit2gtk-4.1-0, libgtk-3-0
Description: TATACOA Usable Alpha desktop QA candidate
 Local-first workspace for authorized security assessments.
EOF
dpkg-deb --root-owner-group --build "$debroot" "$deb"

cliroot="$stage_root/$base-cli"
mkdir -p "$cliroot"
install -m 0755 "$CARGO_TARGET_DIR/release/tatacoa" "$cliroot/tatacoa"
install -m 0755 "$CARGO_TARGET_DIR/release/tatacoa-verify" "$cliroot/tatacoa-verify"
cp LICENSE NOTICE "$cliroot/"
tar --sort=name --mtime="@$epoch" --owner=0 --group=0 --numeric-owner -C "$stage_root" -cf - "$(basename "$cliroot")" | gzip -n > "$out/$base-cli.tar.gz"

artifacts=("$deb" "$out/$base-cli.tar.gz")
[[ ! -f "$out/$base-desktop.AppImage" ]] || artifacts+=("$out/$base-desktop.AppImage")
sha256sum "${artifacts[@]}" | sed "s|$out/||" > "$out/SHA256SUMS.txt"
cat "$out/SHA256SUMS.txt"
