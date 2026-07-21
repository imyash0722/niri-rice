#!/usr/bin/env bash
batt=$(cat /sys/class/power_supply/BAT0/capacity 2>/dev/null || echo "100")
status=$(cat /sys/class/power_supply/BAT0/status 2>/dev/null || echo "Full")

if [ "$status" = "Charging" ]; then
    icon="󰂄"
elif [ "$batt" -ge 90 ]; then icon="󰁹"
elif [ "$batt" -ge 80 ]; then icon="󰂂"
elif [ "$batt" -ge 70 ]; then icon="󰂁"
elif [ "$batt" -ge 60 ]; then icon="󰂀"
elif [ "$batt" -ge 50 ]; then icon="󰁿"
elif [ "$batt" -ge 40 ]; then icon="󰁾"
elif [ "$batt" -ge 30 ]; then icon="󰁽"
elif [ "$batt" -ge 20 ]; then icon="󰁼"
elif [ "$batt" -ge 10 ]; then icon="󰁻"
else icon="󰁺"
fi

echo "$icon $batt%"
