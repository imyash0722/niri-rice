#!/usr/bin/env bash
# Convenience wrapper forwarding to battery-rofi
exec "$(dirname "$0")/battery-rofi" "$@"
