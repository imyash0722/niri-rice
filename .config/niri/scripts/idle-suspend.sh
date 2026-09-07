#!/usr/bin/env bash
# ~/.config/niri/scripts/idle-suspend.sh
# Smart idle suspension logic with exceptions for SSH, Tailscale, AC power, and Caffeine.

# 1. Caffeine Override
if [ -f "/tmp/caffeine_active" ]; then
    exit 0
fi

# 2. Check systemd sleep inhibitors (blocking mode, excluding hypridle)
if systemd-inhibit --list --no-legend 2>/dev/null | grep -i "block" | grep -E "sleep|idle" | grep -qv "hypridle"; then
    exit 0
fi

# 3. Check for AC Power
is_on_ac() {
    for ac in /sys/class/power_supply/AD*/online /sys/class/power_supply/AC*/online /sys/class/power_supply/ucsi-source-psy-*/online; do
        if [ -f "$ac" ] && [ "$(cat "$ac" 2>/dev/null)" = "1" ]; then
            return 0
        fi
    done
    if [ -f "/sys/class/power_supply/BAT0/status" ]; then
        local bat_status
        bat_status="$(cat /sys/class/power_supply/BAT0/status 2>/dev/null)"
        if [ -n "$bat_status" ] && [ "$bat_status" != "Discharging" ]; then
            return 0
        fi
    fi
    return 1
}

# 4. Check for active SSH connections or remote sessions
has_active_ssh() {
    # Check established TCP connections on port 22 (SSH daemon / Tailscale SSH traffic)
    if ss -H -t state established '( sport = :22 )' 2>/dev/null | grep -q .; then
        return 0
    fi
    # Check if sshd has child processes for active client sessions
    if pgrep -f "^sshd: [a-zA-Z0-9]" >/dev/null 2>&1; then
        return 0
    fi
    # Check for active pts sessions connected via remote network
    if who 2>/dev/null | grep -qE "pts/[0-9]+.*\(.*:.*\)|\(100\."; then
        return 0
    fi
    return 1
}

# If on AC power, do NOT suspend.
# Ensure monitors stay powered off and keyboard backlight is off to keep PC cool.
if is_on_ac; then
    niri msg action power-off-monitors 2>/dev/null || true
    brightnessctl -d "*::kbd_backlight" set 0 2>/dev/null || true
    exit 0
fi

# If active SSH / Tailscale session exists (even on battery), do NOT suspend.
if has_active_ssh; then
    niri msg action power-off-monitors 2>/dev/null || true
    brightnessctl -d "*::kbd_backlight" set 0 2>/dev/null || true
    exit 0
fi

# 5. Otherwise, we are on battery with no active SSH connections and no caffeine.
# Suspend system to preserve battery life.
exec systemctl suspend
