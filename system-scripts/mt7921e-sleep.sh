#!/bin/bash
# Reset MT7922 (mt7921e) via direct PCI sysfs unbind/bind.
# This prevents the firmware crash on resume by completely powering
# off the PCIe device state at the lowest kernel level.

PCI_ID="0000:02:00.0"
PCI_DIR="/sys/bus/pci/drivers/mt7921e"

case "$1" in
    pre)
        # Forcefully detach the Wi-Fi card from the kernel driver
        if [ -d "$PCI_DIR" ]; then
            echo "$PCI_ID" > "$PCI_DIR/unbind" 2>/dev/null || true
        fi
        ;;
    post)
        # Reattach the Wi-Fi card to the kernel driver
        if [ -d "$PCI_DIR" ]; then
            echo "$PCI_ID" > "$PCI_DIR/bind" 2>/dev/null || true
        fi
        
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
