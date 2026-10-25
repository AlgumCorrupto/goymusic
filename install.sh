#!/usr/bin/env bash
# Usage:
#   FISH_API_KEY=xxxx ./install.sh     (preferred: key stays out of shell history and `ps`)
#   ./install.sh --fish-key xxxx
#   ./install.sh                       (prompts if the config has no key yet; Enter to skip)
#   ./install.sh --reset-config        (replace the installed config.toml; old one saved as .bak)
#   ./install.sh --uninstall
set -euo pipefail

APP_ID="org.goymusic.Goymusic"   # must match the id registered in hotkeys/portal.rs
APP_NAME="Goymusic"
BIN_NAME="goymusic"

SRC_DIR="$(cd "$(dirname "$0")" && pwd)"
CONFIG_SRC="$SRC_DIR/config.toml"          # the template next to this script

DATA_HOME="${XDG_DATA_HOME:-$HOME/.local/share}"
CONFIG_HOME="${XDG_CONFIG_HOME:-$HOME/.config}"
CARGO_BIN="${CARGO_HOME:-$HOME/.cargo}/bin"

APPS_DIR="$DATA_HOME/applications"
DESKTOP_FILE="$APPS_DIR/$APP_ID.desktop"
CONF_DIR="$CONFIG_HOME/goymusic"
CONFIG_FILE="$CONF_DIR/config.toml"        # where the app looks (Config::default_path("goymusic"))

# Left over from older versions of this script
LEGACY_LAUNCHER="$DATA_HOME/goymusic/launch.sh"

refresh_kde_cache() {
    if command -v kbuildsycoca6 >/dev/null 2>&1; then
        kbuildsycoca6 >/dev/null 2>&1 || true
    elif command -v kbuildsycoca5 >/dev/null 2>&1; then
        kbuildsycoca5 >/dev/null 2>&1 || true
    fi
    command -v update-desktop-database >/dev/null 2>&1 \
        && update-desktop-database "$APPS_DIR" >/dev/null 2>&1 || true
}

# Escapes a value for use inside a TOML basic string ("...").
toml_escape() {
    printf '%s' "$1" | sed 's/\\/\\\\/g; s/"/\\"/g'
}

# True if the installed config already has a non-empty api_key.
has_key() {
    grep -Eq '^api_key[[:space:]]*=[[:space:]]*"[^"]+"' "$CONFIG_FILE" 2>/dev/null
}

# Replaces the api_key line under [tts.fish] and keeps everything else as it is.
# The key goes through the environment, so quotes and backslashes need no awk/sed escaping.
set_api_key() (
    umask 077
    tmp="$CONFIG_FILE.tmp"
    KEY_TOML="$(toml_escape "$1")" awk '
        /^\[/ { in_fish = ($0 == "[tts.fish]") }
        in_fish && /^api_key[ \t]*=/ && !done {
            print "api_key = \"" ENVIRON["KEY_TOML"] "\""
            done = 1
            next
        }
        { print }
        END { exit(done ? 0 : 3) }
    ' "$CONFIG_FILE" > "$tmp" || { rm -f "$tmp"; exit 1; }
    mv "$tmp" "$CONFIG_FILE"
)

# ---- arguments ----
KEY="${FISH_API_KEY:-}"
UNINSTALL=0
RESET_CONFIG=0
while [ $# -gt 0 ]; do
    case "$1" in
        --fish-key)     KEY="${2:?--fish-key needs a value}"; shift 2 ;;
        --fish-key=*)   KEY="${1#*=}"; shift ;;
        --reset-config) RESET_CONFIG=1; shift ;;
        --uninstall)    UNINSTALL=1; shift ;;
        -h|--help)      sed -n '2,7p' "$0"; exit 0 ;;
        *) echo "unknown argument: $1" >&2; exit 1 ;;
    esac
done

if [ "$UNINSTALL" -eq 1 ]; then
    rm -f "$DESKTOP_FILE" "$LEGACY_LAUNCHER"
    rmdir "$(dirname "$LEGACY_LAUNCHER")" 2>/dev/null || true
    cargo uninstall "$BIN_NAME" 2>/dev/null || true
    refresh_kde_cache
    echo "Uninstalled. Your config ($CONFIG_FILE) was left in place; delete it manually if you want."
    exit 0
fi

# ---- build and install the binary ----
command -v cargo >/dev/null 2>&1 || { echo "cargo not found" >&2; exit 1; }
cd "$SRC_DIR"
cargo install --path . --locked --force

BIN="$CARGO_BIN/$BIN_NAME"
[ -x "$BIN" ] || { echo "expected binary at $BIN, but it isn't there" >&2; exit 1; }

# ---- config.toml ----
mkdir -p "$CONF_DIR"
if [ "$RESET_CONFIG" -eq 1 ] || [ ! -f "$CONFIG_FILE" ]; then
    if [ -f "$CONFIG_SRC" ]; then
        if [ -f "$CONFIG_FILE" ]; then
            cp -p "$CONFIG_FILE" "$CONFIG_FILE.bak"
            echo "Old config saved as $CONFIG_FILE.bak"
        fi
        install -m 600 "$CONFIG_SRC" "$CONFIG_FILE"
        echo "Installed config: $CONFIG_FILE"
    else
        echo "warning: $CONFIG_SRC not found; the app will create a config on its first save" >&2
    fi
else
    echo "Keeping the existing config: $CONFIG_FILE (use --reset-config to replace it)"
fi

# ---- API key ----
if [ -z "$KEY" ] && ! has_key && [ -t 0 ]; then
    read -r -s -p "Fish Audio API key (Enter to skip): " KEY
    echo
fi

if [ -n "$KEY" ]; then
    if [ ! -f "$CONFIG_FILE" ]; then
        # No template was available: start a minimal config.
        ( umask 077; printf '[tts.fish]\napi_key = ""\n' > "$CONFIG_FILE" )
    fi
    if set_api_key "$KEY"; then
        echo "Saved API key to $CONFIG_FILE"
    else
        echo "warning: no api_key line under [tts.fish] in $CONFIG_FILE; add it by hand" >&2
    fi
elif has_key; then
    echo "Keeping the API key already in $CONFIG_FILE"
else
    echo "No API key set; you can still type it into the TTS screen."
fi

# ---- desktop entry (file name must equal the app id) ----
rm -f "$LEGACY_LAUNCHER"
rmdir "$(dirname "$LEGACY_LAUNCHER")" 2>/dev/null || true

mkdir -p "$APPS_DIR"
cat > "$DESKTOP_FILE" <<EOF
[Desktop Entry]
Type=Application
Name=$APP_NAME
Comment=Improviser and TTS to a virtual microphone
Exec="$BIN"
Terminal=false
Categories=AudioVideo;Audio;
StartupWMClass=$APP_ID
EOF
chmod 644 "$DESKTOP_FILE"

refresh_kde_cache

echo
echo "Installed:"
echo "  binary   $BIN"
echo "  config   $CONFIG_FILE"
echo "  desktop  $DESKTOP_FILE"
echo "Find \"$APP_NAME\" in the application launcher."