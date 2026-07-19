#!/bin/bash

LOG="/tmp/mt7921e-sleep.log"
echo "[$(date)] Executing $0 with arg: $1" >> $LOG

case "$1" in
    pre)
        # Bring down the interface and unload the driver before sleep
        echo "[$(date)] Suspending: bringing wlan0 down" >> $LOG
        /usr/bin/ip link set wlan0 down 2>>$LOG
        sleep 1
        echo "[$(date)] Unloading mt7921e driver" >> $LOG
        /usr/bin/modprobe -r mt7921e 2>>$LOG
        ;;
    post)
        # Reload the driver after waking up
        echo "[$(date)] Waking up: reloading mt7921e driver" >> $LOG
        /usr/bin/modprobe mt7921e 2>>$LOG
        sleep 1
        echo "[$(date)] Waking up: bringing wlan0 up" >> $LOG
        /usr/bin/ip link set wlan0 up 2>>$LOG
        ;;
esac
