#!/usr/bin/env bash
# ~/.local/bin/weather.sh — Lightweight weather module for Waybar

# Fetch weather (format: emoji condition + temp, e.g. "⛅ +28°C")
WEATHER=$(curl -s --connect-timeout 2 "https://wttr.in/?format=%c+%t" 2>/dev/null | tr -s ' ' | sed 's/^[ \t]*//;s/[ \t]*$//')

if [ -n "$WEATHER" ] && [[ ! "$WEATHER" =~ "Unknown" ]] && [[ ! "$WEATHER" =~ "html" ]] && [[ ! "$WEATHER" =~ "Error" ]]; then
    # Output Waybar JSON format
    printf '{"text": "%s", "tooltip": "Weather: %s\\nSource: wttr.in", "class": "weather"}\n' "$WEATHER" "$WEATHER"
else
    # Fallback when offline or timeout
    printf '{"text": "⛅ --°C", "tooltip": "Weather: Offline / Waiting for connection", "class": "weather-offline"}\n'
fi
