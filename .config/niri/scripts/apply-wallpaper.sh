#!/bin/bash
set -e

# ==============================================================================
# Universal Wallpaper Processor & Applicator (Niri & KDE Plasma)
# Features:
#   - Scales and center-crops ANY resolution/aspect ratio to exactly 1920x1200
#   - Converts GIFs/Videos to smooth 1200p hardware-accelerated MP4
#   - Converts static images to 1920x1200 PNG
#   - Automatically extracts Pywal colors
#   - Automatically syncs matching 24px Moga-Neon cursor in Niri & KDE
#   - Applies live wallpaper in Niri (mpvpaper) and KDE Plasma
# ==============================================================================

INPUT="$1"
OUTPUT_NAME="${2:-custom}"
TARGET_W=1920
TARGET_H=1200

if [ -z "$INPUT" ] || [ ! -f "$INPUT" ]; then
    echo "Usage: $0 <path-to-image-or-video-or-gif> [theme-or-output-name]"
    echo "Example: $0 ~/Downloads/cool_art.gif cyber"
    exit 1
fi

THEMES_DIR="$HOME/.config/niri/themes"
DEST_DIR="$THEMES_DIR/$OUTPUT_NAME"
mkdir -p "$DEST_DIR"

MIME=$(file --mime-type -b "$INPUT")
echo "[1/4] Processing input '$INPUT' (type: $MIME)..."

IS_ANIMATED=false
if [[ "$MIME" =~ ^video/ ]] || [[ "$MIME" == "image/gif" ]] || [[ "$INPUT" =~ \.(mp4|mkv|webm|mov|avi|gif)$ ]]; then
    IS_ANIMATED=true
fi

OUT_MP4="$DEST_DIR/wallpaper.mp4"
OUT_PNG="$DEST_DIR/wallpaper.png"

if [ "$IS_ANIMATED" = true ]; then
    echo "  -> Converting animated input to 1920x1200 H.264 MP4..."
    ffmpeg -y -i "$INPUT" \
        -vf "scale=${TARGET_W}:${TARGET_H}:force_original_aspect_ratio=increase,crop=${TARGET_W}:${TARGET_H}" \
        -c:v libx264 -pix_fmt yuv420p -profile:v high -level:v 4.2 -crf 18 -preset slow -an \
        -movflags +faststart "$OUT_MP4" 2>/dev/null
    
    # Extract pristine static PNG preview frame for KDE image mode
    ffmpeg -y -ss 00:00:01 -i "$OUT_MP4" -vframes 1 -f image2 "$OUT_PNG" 2>/dev/null || \
    ffmpeg -y -i "$OUT_MP4" -vframes 1 -f image2 "$OUT_PNG" 2>/dev/null
else
    echo "  -> Converting static image to 1920x1200 PNG..."
    ffmpeg -y -i "$INPUT" \
        -vf "scale=${TARGET_W}:${TARGET_H}:force_original_aspect_ratio=increase,crop=${TARGET_W}:${TARGET_H}" \
        "$OUT_PNG" 2>/dev/null
    
    # Also generate a lightweight 1-frame MP4 for mpvpaper seamless looping
    ffmpeg -y -loop 1 -i "$OUT_PNG" -t 1 -c:v libx264 -pix_fmt yuv420p -crf 18 -preset fast -an "$OUT_MP4" 2>/dev/null
fi

echo "[2/4] Extracting Pywal dynamic color palette..."
rm -rf "$HOME/.cache/wal/schemes"
wal -i "$OUT_PNG" -n -s -t -q

echo "[3/4] Auto-matching & applying 24px Moga-Neon cursor and desktop themes..."
if [ -x "$HOME/.config/niri/scripts/pywal-cursor.py" ]; then
    "$HOME/.config/niri/scripts/pywal-cursor.py" --wallpaper "$OUT_PNG" --size 32
fi

echo "[4/4] Setting active wallpaper for Niri and KDE Plasma..."
# 1. Update Niri active wallpaper path
echo "$OUT_MP4" > "$THEMES_DIR/active-wallpaper.txt"

# 2. Update KDE Plasma wallpaper
if command -v plasma-apply-wallpaperimage &>/dev/null && [ -f "$OUT_PNG" ]; then
    plasma-apply-wallpaperimage "$OUT_PNG" &>/dev/null || true
fi
if command -v kwriteconfig6 &>/dev/null; then
    kwriteconfig6 --file plasma-org.kde.plasma.desktop-appletsrc --group Containments --group 210 --group Wallpaper --group "luisbocanegra.smart.video.wallpaper.reborn" --group General --key Video "file://$OUT_MP4" 2>/dev/null || true
fi

# 3. Apply with droplet bubble transition via awww
if [ -n "$NIRI_SOCKET" ]; then
    # Ensure awww-daemon is active
    pgrep -x awww-daemon >/dev/null || nohup awww-daemon >/dev/null 2>&1 &
    
    # Trigger circular droplet bubble animation expanding from center
    awww img "$OUT_PNG" \
        --transition-type center \
        --transition-pos center \
        --transition-duration 1.2 \
        --transition-fps 120 \
        --transition-bezier .25,1,.5,1 2>/dev/null || true

    if [ "$IS_ANIMATED" = true ]; then
        # For animated MP4 wallpapers, let the droplet bubble finish then seamlessly start mpvpaper
        (
            sleep 1.2
            pkill -9 -x mpvpaper 2>/dev/null || true
            nohup mpvpaper -o 'no-audio loop-file=inf hwdec=vaapi msg-level=all=error video-unscaled=yes' '*' "$OUT_MP4" >/dev/null 2>&1 &
        ) &
    else
        # For static image wallpapers, dismiss mpvpaper so awww displays the static frame directly
        pkill -9 -x mpvpaper 2>/dev/null || true
    fi
fi

echo "All done! 1920x1200 wallpaper, Pywal palette, and 24px cursor successfully applied!"
