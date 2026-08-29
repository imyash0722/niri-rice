#!/bin/bash
# Reset MT7922 (mt7921e) via PCI Hot-Unplug / Rescan
# This fully physically cuts the device from the PCI bus before sleep
# and forces a cold boot of the Wi-Fi card after wake.

PCI_DEV="/sys/bus/pci/devices/0000:02:00.0"

case "$1" in
    pre)
        # Forcefully hot-unplug the Wi-Fi card from the PCI bus
        if [ -d "$PCI_DEV" ]; then
            echo "1" > "$PCI_DEV/remove" 2>/dev/null || true
        fi
        ;;
    post)
        # Rescan the PCI bus to hot-plug the card back in
        echo "1" > /sys/bus/pci/rescan 2>/dev/null || true
        
        # Wait for the device to initialize and the interface to appear
        for i in {1..10}; do
            IFACE=$(ip link show | grep -oP '^[0-9]+: \Kwl[^:]+')
            if [ -n "$IFACE" ]; then
                break
            fi
            sleep 0.5
        done
        
        if [ -n "$IFACE" ]; then
            # The MT7922 has a known firmware bug where hardware power saving
            # causes stuttering/disconnects. Disable it immediately.
            iw dev "$IFACE" set power_save off 2>/dev/null || true
        fi
        ;;
esac
exit 0
