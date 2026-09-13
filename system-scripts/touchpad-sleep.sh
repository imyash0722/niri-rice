#!/bin/bash
# Re-apply runtime power control and restore AMD I2C / ELAN touchpad after sleep/wake
case "$1" in
    post)
        /usr/local/bin/fix-touchpad-power.sh
        ;;
esac
exit 0
