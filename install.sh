#!/usr/bin/env bash
# Usage:
#   FISH_API_KEY=xxxx ./install.sh     (preferred: key stays out of shell history and `ps`)
#   ./install.sh --fish-key xxxx
#   ./install.sh                       (prompts if no key is stored yet; Enter to skip)
#   ./install.sh --uninstall
set -euo pipefail

APP_ID="org.goymusic.Goymusic"   # must match the id registered in hotkeys/portal.rs
APP_NAME="Goymusic"
BIN_NAME="goymusic"

DATA_HOME="${XDG_DATA_HOME:-$HOME/.local/share}"
CONFIG_HOME="${XDG_CONFIG_HOME:-$HOME/.config}"
CARGO_BIN="${CARGO_HOME:-$HOME/.cargo}/bin"

APPS_DIR="$DATA_HOME/applications"
DESKTOP_FILE="$APPS_DIR/$APP_ID.desktop"
APP_DATA="$DATA_HOME/goymusic"
LAUNCHER="$APP_DATA/launch.sh"
CONF_DIR="$CONFIG_HOME/goymusic"
ENV_FILE="$CONF_DIR/env"

refresh_kde_cache() {
    if command -v kbuildsycoca6 >/dev/null 2>&1; then
        kbuildsycoca6 >/dev/null 2>&1 || true
    elif command -v kbuildsycoca5 >/dev/null 2>&1; then
        kbuildsycoca5 >/dev/null 2>&1 || true
    fi
    command -v update-desktop-database >/dev/null 2>&1 \
        && update-desktop-database "$APPS_DIR" >/dev/null 2>&1 || true
}

# ---- arguments ----
KEY="${FISH_API_KEY:-}"
UNINSTALL=0
while [ $# -gt 0 ]; do
    case "$1" in
        --fish-key)   KEY="${2:?--fish-key needs a value}"; shift 2 ;;
        --fish-key=*) KEY="${1#*=}"; shift ;;
        --uninstall)  UNINSTALL=1; shift ;;
        -h|--help)    sed -n '2,7p' "$0"; exit 0 ;;
        *) echo "unknown argument: $1" >&2; exit 1 ;;
    esac
done

if [ "$UNINSTALL" -eq 1 ]; then
    rm -f "$DESKTOP_FILE" "$LAUNCHER"
    rmdir "$APP_DATA" 2>/dev/null || true
    cargo uninstall "$BIN_NAME" 2>/dev/null || true
    refresh_kde_cache
    echo "Uninstalled. Your key file ($ENV_FILE) was left in place; delete it manually if you want."
    exit 0
fi

# ---- build and install the binary ----
command -v cargo >/dev/null 2>&1 || { echo "cargo not found" >&2; exit 1; }
cd "$(dirname "$0")"
cargo install --path . --locked --force

BIN="$CARGO_BIN/$BIN_NAME"
[ -x "$BIN" ] || { echo "expected binary at $BIN, but it isn't there" >&2; exit 1; }

# ---- API key ----
mkdir -p "$CONF_DIR"
if [ -z "$KEY" ] && [ ! -f "$ENV_FILE" ] && [ -t 0 ]; then
    read -r -s -p "Fish Audio API key (Enter to skip): " KEY
    echo
fi

if [ -n "$KEY" ]; then
    umask 077
    printf 'export FISH_API_KEY=%q\n' "$KEY" > "$ENV_FILE"
    chmod 600 "$ENV_FILE"
    echo "Saved API key to $ENV_FILE"
elif [ -f "$ENV_FILE" ]; then
    echo "Keeping the existing key in $ENV_FILE"
else
    echo "No API key set; you can still type it into the TTS screen."
fi

# ---- launcher wrapper: loads the key, then replaces itself with the app ----
mkdir -p "$APP_DATA"
cat > "$LAUNCHER" <<EOF
#!/bin/sh
[ -f "$ENV_FILE" ] && . "$ENV_FILE"
exec "$BIN" "\$@"
EOF
chmod 755 "$LAUNCHER"

# ---- desktop entry (file name must equal the app id) ----
mkdir -p "$APPS_DIR"
cat > "$DESKTOP_FILE" <<EOF
[Desktop Entry]
Type=Application
Name=$APP_NAME
Comment=Improviser and TTS to a virtual microphone
Exec=$LAUNCHER
Terminal=false
Categories=AudioVideo;Audio;
StartupWMClass=$APP_ID
EOF
chmod 644 "$DESKTOP_FILE"

refresh_kde_cache

echo
echo "Installed:"
echo "  binary   $BIN"
echo "  launcher $LAUNCHER"
echo "  desktop  $DESKTOP_FILE"
echo "Find \"$APP_NAME\" in the application launcher."