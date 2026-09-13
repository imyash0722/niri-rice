#!/usr/bin/env bash
# /usr/local/bin/fix-touchpad-power.sh
# Fix Lenovo ThinkBook 16 G7 AMD I2C / ELAN Touchpad power management & freezing
set -euo pipefail

for dev in /sys/devices/platform/AMDI0010:01 /sys/bus/i2c/devices/*ELAN* /sys/bus/i2c/devices/*AMDI0010*; do
    if [ -d "$dev" ]; then
        find "$dev" -name "control" -exec sh -c 'for f; do [ -f "$f" ] && echo on > "$f" 2>/dev/null || true; done' _ {} +
    fi
done

if [ "${1:-}" = "--rebind" ] || [ "${1:-}" = "rebind" ]; then
    if [ -e /sys/bus/i2c/drivers/i2c_hid_acpi/i2c-ELAN06FA:00 ]; then
        echo "i2c-ELAN06FA:00" > /sys/bus/i2c/drivers/i2c_hid_acpi/unbind
        sleep 0.5
        echo "i2c-ELAN06FA:00" > /sys/bus/i2c/drivers/i2c_hid_acpi/bind
        sleep 0.5
        for dev in /sys/devices/platform/AMDI0010:01 /sys/bus/i2c/devices/*ELAN*; do
            [ -d "$dev" ] && find "$dev" -name "control" -exec sh -c 'for f; do [ -f "$f" ] && echo on > "$f" 2>/dev/null || true; done' _ {} +
        done
    fi
fi
