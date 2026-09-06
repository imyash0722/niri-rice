#!/usr/bin/env bash
# ~/.local/bin/wifi.sh — Network Management Rofi Applet

ROFI_CMD="rofi -dmenu -i -theme-str 'window { width: 480px; location: center; anchor: center; } listview { lines: 6; }'"

ICON_DIR="$HOME/.icons/icon_scripts/dunst/wifi"

notify() {
    local message="$1"
    local kind="${2:-info}"
    local icon="$ICON_DIR/${kind}.svg"

    if command -v dunstify &>/dev/null; then
        dunstify -i "$icon" "Network: $message" -a network
    fi
}

active_connections() {
    nmcli -t -f NAME,TYPE,DEVICE con show --active 2>/dev/null |
        grep -v ":loopback"
}

scan_and_select() {
    python3 - << 'PYEOF'
import subprocess
import sys

res = subprocess.run(
    ["nmcli", "-t", "-f", "IN-USE,SIGNAL,SECURITY,SSID", "dev", "wifi", "list"],
    capture_output=True, text=True
)

networks = {}
for line in res.stdout.strip().split("\n"):
    if not line:
        continue
    parts = line.split(":")
    if len(parts) < 4:
        continue
    in_use = parts[0].strip() == "*"
    try:
        signal = int(parts[1].strip())
    except ValueError:
        signal = 0
    security = parts[2].strip()
    ssid = ":".join(parts[3:]).strip()
    if not ssid or ssid == "--":
        continue

    if ssid not in networks:
        networks[ssid] = {"in_use": in_use, "signal": signal, "security": security}
    else:
        if in_use:
            networks[ssid]["in_use"] = True
        if signal > networks[ssid]["signal"]:
            networks[ssid]["signal"] = signal
            if security and security != "--":
                networks[ssid]["security"] = security

if not networks:
    sys.exit(2)

# Sort: connected first, then highest signal
sorted_nets = sorted(networks.items(), key=lambda x: (x[1]["in_use"], x[1]["signal"]), reverse=True)
options = []
for ssid, info in sorted_nets:
    sec_icon = "" if (info["security"] and info["security"] != "--") else " "
    sig = info["signal"]
    if sig >= 75:
        sig_icon = "󰤨"
    elif sig >= 50:
        sig_icon = "󰤥"
    elif sig >= 25:
        sig_icon = "󰤢"
    else:
        sig_icon = "󰤟"
    status_tag = "   Connected" if info["in_use"] else ""
    options.append(f"{sig_icon}  {ssid:<26} {sig:>3}%  {sec_icon}{status_tag}")

rofi_cmd = [
    "rofi", "-dmenu", "-i", "-format", "i",
    "-theme-str", "window { width: 560px; location: center; anchor: center; } listview { lines: 10; }",
    "-p", "Connect to Wi-Fi"
]
rofi_proc = subprocess.run(rofi_cmd, input="\n".join(options), text=True, capture_output=True)
if rofi_proc.returncode != 0 or not rofi_proc.stdout.strip():
    sys.exit(1)

try:
    chosen_idx = int(rofi_proc.stdout.strip())
    chosen_ssid, info = sorted_nets[chosen_idx]
    print(f"{chosen_ssid}\t{info['security']}")
except Exception:
    sys.exit(1)
PYEOF
}

do_connect() {
    nmcli dev wifi rescan &>/dev/null &

    local selection
    selection=$(scan_and_select)
    local ret=$?

    if [[ $ret -eq 2 ]]; then
        notify "No Wi-Fi networks found" info
        return
    elif [[ $ret -ne 0 || -z "$selection" ]]; then
        return
    fi

    local chosen security
    IFS=$'\t' read -r chosen security <<< "$selection"

    [[ -z "$chosen" ]] && return

    # Check if network profile already exists
    if nmcli -t -f NAME con show | grep -Fxq "$chosen"; then
        if nmcli con up "$chosen" &>/dev/null; then
            notify "Connected to $chosen" connected
        else
            notify "Failed to connect to $chosen" error
        fi
        return
    fi

    # Prompt for password if secured
    local password=""
    if [[ "$security" != "--" && "$security" != "" ]]; then
        password=$(rofi -dmenu -password \
            -theme-str 'window { width: 440px; location: center; anchor: center; }' \
            -p "Password for $chosen")
        [[ -z "$password" ]] && return
    fi

    if [[ -n "$password" ]]; then
        if nmcli dev wifi connect "$chosen" password "$password" &>/dev/null; then
            notify "Connected to $chosen" connected
        else
            notify "Failed to connect to $chosen" error
        fi
    else
        if nmcli dev wifi connect "$chosen" &>/dev/null; then
            notify "Connected to $chosen" connected
        else
            notify "Failed to connect to $chosen" error
        fi
    fi
}

do_disconnect() {
    local connections
    connections=$(active_connections)

    if [[ -z "$connections" ]]; then
        notify "No active connections" info
        return
    fi

    local chosen
    chosen=$(echo "$connections" |
        awk -F: '{print "󰖩  " $1 "  (" $3 ")"}' |
        rofi -dmenu -i \
            -theme-str 'window { width: 480px; location: center; anchor: center; } listview { lines: 5; }' \
            -p 'Disconnect')

    [[ -z "$chosen" ]] && return

    local conn_name
    conn_name=$(echo "$chosen" | sed -E 's/^󰖩  //; s/  \(.*\)$//')

    if nmcli con down "$conn_name" &>/dev/null; then
        notify "Disconnected from $conn_name" disconnected
    else
        notify "Failed to disconnect $conn_name" error
    fi
}

do_status() {
    local active wifi_status conn_state ip_addr
    active=$(active_connections | awk -F: 'NR==1{print $1}')
    [[ -z "$active" ]] && active="Disconnected"

    wifi_status=$(nmcli -t -f WIFI g 2>/dev/null || nmcli radio wifi 2>/dev/null | tail -n 1)
    conn_state=$(nmcli -t -f CONNECTIVITY g 2>/dev/null)
    [[ -z "$conn_state" ]] && conn_state="unknown"

    ip_addr=$(ip -4 route get 1.1.1.1 2>/dev/null | awk '{for(i=1;i<=NF;i++) if($i=="src") print $(i+1)}')
    [[ -z "$ip_addr" ]] && ip_addr="Not Assigned"

    printf "󰖩  Network:       %s\n󰩠  IPv4 Address:  %s\n󰈀  Connectivity:  %s\n󰤨  Wi-Fi Radio:   %s\n" \
        "$active" "$ip_addr" "$conn_state" "$wifi_status" |
        rofi -dmenu -p "Network Status" \
            -theme-str 'window { width: 480px; location: center; anchor: center; } listview { lines: 4; } entry { enabled: false; }' \
            >/dev/null
}

do_toggle_wifi() {
    local current
    current=$(nmcli radio wifi 2>/dev/null | tail -n +2 | awk '{print $1}')
    [[ -z "$current" ]] && current=$(nmcli -t -f WIFI g 2>/dev/null)

    if [[ "$current" == "enabled" ]]; then
        nmcli radio wifi off && notify "Wi-Fi disabled" disconnected
    else
        nmcli radio wifi on && notify "Wi-Fi enabled" connected
    fi
}

do_toggle_ethernet() {
    local devices
    devices=$(nmcli -t -f DEVICE,TYPE,STATE dev status 2>/dev/null |
        awk -F: '$2=="ethernet"{print $1":"$3}')

    if [[ -z "$devices" ]]; then
        notify "No ethernet devices found" info
        return
    fi

    local dev_count
    dev_count=$(echo "$devices" | grep -c .)

    local chosen_dev chosen_state
    if [[ "$dev_count" -eq 1 ]]; then
        chosen_dev=$(echo "$devices" | cut -d: -f1)
        chosen_state=$(echo "$devices" | cut -d: -f2)
    else
        local chosen
        chosen=$(echo "$devices" |
            awk -F: '{print "󰈀  " $1 " (" $2 ")"}' |
            rofi -dmenu -i \
                -theme-str 'window { width: 480px; location: center; anchor: center; } listview { lines: 5; }' \
                -p 'Toggle Ethernet')

        [[ -z "$chosen" ]] && return

        chosen_dev=$(echo "$chosen" | awk '{print $2}')
        chosen_state=$(echo "$devices" | awk -F: -v d="$chosen_dev" '$1==d{print $2}')
    fi

    if [[ "$chosen_state" == "unavailable" || "$chosen_state" == "unmanaged" ]]; then
        notify "Ethernet device $chosen_dev is $chosen_state" error
        return
    fi

    if [[ "$chosen_state" == "disconnected" ]]; then
        if nmcli dev connect "$chosen_dev" &>/dev/null; then
            notify "Ethernet ($chosen_dev) enabled" connected
        else
            notify "Failed to enable ethernet ($chosen_dev)" error
        fi
    else
        if nmcli dev disconnect "$chosen_dev" &>/dev/null; then
            notify "Ethernet ($chosen_dev) disabled" disconnected
        else
            notify "Failed to disable ethernet ($chosen_dev)" error
        fi
    fi
}

do_vpn() {
    local vpns
    vpns=$(nmcli -t -f NAME,TYPE con show |
        awk -F: '$2~/vpn|wireguard/{print $1}')

    if [[ -z "$vpns" ]]; then
        notify "No VPN connections configured" info
        return
    fi

    local chosen
    chosen=$(echo "$vpns" |
        awk '{print "󰖂  " $0}' |
        rofi -dmenu -i \
            -theme-str 'window { width: 480px; location: center; anchor: center; } listview { lines: 6; }' \
            -p 'Toggle VPN')

    [[ -z "$chosen" ]] && return
    chosen=$(echo "$chosen" | sed 's/^󰖂  //')

    if nmcli -t -f NAME,TYPE con show --active | grep -q "^$chosen:"; then
        nmcli con down "$chosen" && notify "VPN $chosen disconnected" disconnected
    else
        nmcli con up "$chosen" && notify "VPN $chosen connected" connected ||
            notify "Failed to start VPN $chosen" error
    fi
}

MENU="󰤨  Connect to Wi-Fi\n󰤭  Disconnect\n󰖩  Network Status\n󰤨  Toggle Wi-Fi\n󰈀  Toggle Ethernet\n󰖂  VPN"

choice=$(printf "$MENU" |
    rofi -dmenu -i \
        -theme-str 'window { width: 480px; location: center; anchor: center; } listview { lines: 6; }' \
        -p 'Network')

case "$choice" in
*"Connect to Wi-Fi") do_connect ;;
*"Disconnect") do_disconnect ;;
*"Network Status") do_status ;;
*"Toggle Wi-Fi") do_toggle_wifi ;;
*"Toggle Ethernet") do_toggle_ethernet ;;
*"VPN") do_vpn ;;
*) exit 0 ;;
esac
