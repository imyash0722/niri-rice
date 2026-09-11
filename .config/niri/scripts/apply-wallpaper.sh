#!/bin/bash
# Native Rust Rice-Ctl Wallpaper & Theming Applicator
set -e

INPUT="$1"
if [ -z "$INPUT" ] || [ ! -f "$INPUT" ]; then
    echo "Usage: $0 <path-to-wallpaper>"
    exit 1
fi

exec rice-ctl theme set "$INPUT"
