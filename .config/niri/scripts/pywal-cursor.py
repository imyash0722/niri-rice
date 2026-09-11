#!/bin/bash
# Backward-compatibility forwarder to native Rust rice-ctl
WALLPAPER=""
SIZE="32"

while [[ $# -gt 0 ]]; do
  case $1 in
    --wallpaper)
      WALLPAPER="$2"
      shift 2
      ;;
    --size)
      SIZE="$2"
      shift 2
      ;;
    *)
      shift
      ;;
  esac
done

if [ -n "$WALLPAPER" ]; then
  exec rice-ctl theme set "$WALLPAPER" --size "$SIZE"
else
  exec rice-ctl theme random --size "$SIZE"
fi
