#!/bin/bash
ANIM_DIR="$HOME/.config/niri/animations"
ANIM="$1"

if [ -z "$ANIM" ]; then
    echo "Usage: $0 <animation-name>"
    echo ""
    echo "Available animations:"
    for f in "$ANIM_DIR"/*.kdl; do
        basename "$f" .kdl
    done
    exit 1
fi

if [ ! -f "$ANIM_DIR/$ANIM.kdl" ]; then
    echo "Error: Animation '$ANIM' not found in $ANIM_DIR"
    echo "Available animations:"
    for f in "$ANIM_DIR"/*.kdl; do
        basename "$f" .kdl
    done
    exit 1
fi

echo "Setting animation: $ANIM..."
echo "include \"../animations/$ANIM.kdl\"" > "$HOME/.config/niri/themes/active-animations.kdl"

if [ -n "$NIRI_SOCKET" ]; then
    niri msg action do-screen-transition 2>/dev/null || true
    echo "Animation '$ANIM' active live in Niri!"
else
    echo "Animation '$ANIM' set! Will apply on next Niri launch or Mod+Shift+R."
fi
