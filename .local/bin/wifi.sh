#!/usr/bin/env bash
# ~/.local/bin/wifi.sh — Network & VPN Management Rofi Applet

ROFI_CMD="rofi -dmenu -i -theme-str 'window { width: 380px; location: center; anchor: center; } listview { lines: 7; }'"

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

sorted_nets = sorted(networks.items(), key=lambda x: (x[1]["in_use"], x[1]["signal"]), reverse=True)
options = []
for ssid, info in sorted_nets:
    sec_icon = " " if (info["security"] and info["security"] != "--") else " "
    sig = info["signal"]
    if sig >= 75:
        sig_icon = "󰤨"
    elif sig >= 50:
        sig_icon = "󰤥"
    elif sig >= 25:
        sig_icon = "󰤢"
    else:
        sig_icon = "󰤟"
    status_tag = " " if info["in_use"] else ""
    options.append(f"{sig_icon} {ssid} · {sig}%{sec_icon}{status_tag}")

rofi_cmd = [
    "rofi", "-dmenu", "-i", "-format", "i",
    "-theme-str", "window { width: 420px; location: center; anchor: center; } listview { lines: 10; }",
    "-p", "Connect"
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
            -theme-str 'window { width: 360px; location: center; anchor: center; }' \
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
        awk -F: '{print "󰖩 " $1 " (" $3 ")"}' |
        rofi -dmenu -i \
            -theme-str 'window { width: 380px; location: center; anchor: center; } listview { lines: 5; }' \
            -p 'Disconnect')

    [[ -z "$chosen" ]] && return

    local conn_name
    conn_name=$(echo "$chosen" | sed -E 's/^󰖩 //; s/ \(.*\)$//')

    if nmcli con down "$conn_name" &>/dev/null; then
        notify "Disconnected from $conn_name" disconnected
    else
        notify "Failed to disconnect $conn_name" error
    fi
}

do_toggle_dns() {
    local new_state
    new_state=$(sudo -n /usr/local/bin/cloudflare-dns toggle 2>/dev/null)
    if [[ "$new_state" == "enabled" ]]; then
        notify "Cloudflare DNS (1.1.1.1) Enabled (DoT)" connected
    else
        notify "Cloudflare DNS Disabled (Default network DNS)" disconnected
    fi
}

get_active_vpn_name() {
    if warp-cli status 2>/dev/null | grep -q "Connected" && ! warp-cli status 2>/dev/null | grep -q "Disconnected"; then
        echo "Cloudflare WARP"
        return
    fi
    if protonvpn status 2>/dev/null | grep -iq "connected" && ! protonvpn status 2>/dev/null | grep -iq "disconnected"; then
        echo "Proton VPN"
        return
    fi
    local ts_exit
    ts_exit=$(tailscale status --json 2>/dev/null | jq -r '.ExitNodeStatus.TailscaleIPs[0] // empty' 2>/dev/null)
    if [[ -n "$ts_exit" ]]; then
        echo "Tailscale ($ts_exit)"
        return
    fi
    local nm_vpn
    nm_vpn=$(nmcli -t -f NAME,TYPE con show --active 2>/dev/null | awk -F: '$2~/vpn|wireguard/{print $1}' | head -n 1)
    if [[ -n "$nm_vpn" ]]; then
        echo "NM: $nm_vpn"
        return
    fi
    echo "Direct"
}

do_status() {
    local active wifi_status conn_state ip_addr dns_state vpn_state
    active=$(active_connections | awk -F: 'NR==1{print $1}')
    [[ -z "$active" ]] && active="Disconnected"

    wifi_status=$(nmcli -t -f WIFI g 2>/dev/null || nmcli radio wifi 2>/dev/null | tail -n 1)
    conn_state=$(nmcli -t -f CONNECTIVITY g 2>/dev/null)
    [[ -z "$conn_state" ]] && conn_state="unknown"

    ip_addr=$(ip -4 route get 1.1.1.1 2>/dev/null | awk '{for(i=1;i<=NF;i++) if($i=="src") print $(i+1)}')
    [[ -z "$ip_addr" ]] && ip_addr="None"

    dns_state=$(sudo -n /usr/local/bin/cloudflare-dns status 2>/dev/null)
    if [[ "$dns_state" == "enabled" ]]; then
        dns_desc="1.1.1.1 (DoT)"
    else
        dns_desc="Default (DHCP)"
    fi

    vpn_state=$(get_active_vpn_name)

    printf "󰖩 Network: %s\n󰩠 IPv4: %s\n󰈀 State: %s\n󰤨 Radio: %s\n󰒍 DNS: %s\n󰖂 VPN: %s\n" \
        "$active" "$ip_addr" "$conn_state" "$wifi_status" "$dns_desc" "$vpn_state" |
        rofi -dmenu -p "Status" \
            -theme-str 'window { width: 380px; location: center; anchor: center; } listview { lines: 6; } entry { enabled: false; }' \
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
            awk -F: '{print "󰈀 " $1 " (" $2 ")"}' |
            rofi -dmenu -i \
                -theme-str 'window { width: 380px; location: center; anchor: center; } listview { lines: 5; }' \
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

do_vpn_warp() {
    if warp-cli status 2>/dev/null | grep -q "Connected" && ! warp-cli status 2>/dev/null | grep -q "Disconnected"; then
        warp-cli disconnect &>/dev/null
        notify "Cloudflare WARP disconnected" disconnected
    else
        warp-cli connect &>/dev/null
        notify "Cloudflare WARP connected" connected
    fi
}

do_vpn_tailscale() {
    local is_exit
    is_exit=$(tailscale status --json 2>/dev/null | jq -r '.ExitNodeStatus // empty')
    if [[ -n "$is_exit" ]]; then
        tailscale up --exit-node="" &>/dev/null
        notify "Tailscale Exit Node disabled" disconnected
    else
        tailscale up --exit-node=pineapple-station --exit-node-allow-lan-access &>/dev/null
        notify "Tailscale routed via pineapple-station" connected
    fi
}

do_vpn_proton() {
    if protonvpn status 2>/dev/null | grep -iq "connected" && ! protonvpn status 2>/dev/null | grep -iq "disconnected"; then
        protonvpn disconnect &>/dev/null
        notify "Proton VPN disconnected" disconnected
    else
        notify "Connecting to Proton VPN..." info
        if protonvpn connect &>/dev/null; then
            notify "Proton VPN connected" connected
        else
            notify "Proton VPN connection failed" error
        fi
    fi
}

do_vpn_nm() {
    local vpns
    vpns=$(nmcli -t -f NAME,TYPE con show | awk -F: '$2~/vpn|wireguard/{print $1}')

    if [[ -z "$vpns" ]]; then
        notify "No custom VPN profiles found" info
        return
    fi

    local chosen
    chosen=$(echo "$vpns" |
        awk '{print "󰒍 " $0}' |
        rofi -dmenu -i \
            -theme-str 'window { width: 380px; location: center; anchor: center; } listview { lines: 6; }' \
            -p 'Custom VPN')

    [[ -z "$chosen" ]] && return
    chosen=$(echo "$chosen" | sed 's/^󰒍 //')

    if nmcli -t -f NAME,TYPE con show --active | grep -q "^$chosen:"; then
        nmcli con down "$chosen" && notify "VPN $chosen disconnected" disconnected
    else
        nmcli con up "$chosen" && notify "VPN $chosen connected" connected ||
            notify "Failed to start VPN $chosen" error
    fi
}

do_vpn_disconnect_all() {
    warp-cli disconnect &>/dev/null
    protonvpn disconnect &>/dev/null
    tailscale up --exit-node="" &>/dev/null
    local nm_active
    nm_active=$(nmcli -t -f NAME,TYPE con show --active 2>/dev/null | awk -F: '$2~/vpn|wireguard/{print $1}')
    for v in $nm_active; do
        nmcli con down "$v" &>/dev/null
    done
    notify "All VPNs disconnected" disconnected
}

do_vpn_switcher() {
    local warp_state ts_state pvpn_state nm_count
    
    if warp-cli status 2>/dev/null | grep -q "Connected" && ! warp-cli status 2>/dev/null | grep -q "Disconnected"; then
        warp_state="Active"
    else
        warp_state="Off"
    fi

    local ts_exit
    ts_exit=$(tailscale status --json 2>/dev/null | jq -r '.ExitNodeStatus.TailscaleIPs[0] // empty' 2>/dev/null)
    if [[ -n "$ts_exit" ]]; then
        ts_state="Active (Station)"
    else
        ts_state="Off"
    fi

    if protonvpn status 2>/dev/null | grep -iq "connected" && ! protonvpn status 2>/dev/null | grep -iq "disconnected"; then
        pvpn_state="Active"
    else
        pvpn_state="Off"
    fi

    nm_count=$(nmcli -t -f NAME,TYPE con show 2>/dev/null | awk -F: '$2~/vpn|wireguard/{print $1}' | wc -l)

    local vpn_menu
    vpn_menu="󰅠 Cloudflare WARP [$warp_state]\n󰋜 Tailscale Gateway [$ts_state]\n󰖂 Proton VPN (Stealth) [$pvpn_state]\n󰒍 Custom Profiles ($nm_count)\n󰅖 Disconnect All"

    local choice
    choice=$(printf "$vpn_menu" |
        rofi -dmenu -i \
            -theme-str 'window { width: 400px; location: center; anchor: center; } listview { lines: 5; }' \
            -p 'VPN')

    case "$choice" in
    *"Cloudflare WARP"*) do_vpn_warp ;;
    *"Tailscale Gateway"*) do_vpn_tailscale ;;
    *"Proton VPN"*) do_vpn_proton ;;
    *"Custom Profiles"*) do_vpn_nm ;;
    *"Disconnect All"*) do_vpn_disconnect_all ;;
    *) return ;;
    esac
}

# --- Main Applet Menu ---
dns_state=$(sudo -n /usr/local/bin/cloudflare-dns status 2>/dev/null)
if [[ "$dns_state" == "enabled" ]]; then
    dns_badge="[Active]"
else
    dns_badge="[Off]"
fi

MENU="󰤨 Connect to Wi-Fi\n󰤭 Disconnect\n󰖩 Network Status\n󰖂 VPN Switcher\n󰒍 Cloudflare DNS $dns_badge\n󰤨 Toggle Wi-Fi\n󰈀 Toggle Ethernet"

choice=$(printf "$MENU" |
    rofi -dmenu -i \
        -theme-str 'window { width: 380px; location: center; anchor: center; } listview { lines: 7; }' \
        -p 'Network')

case "$choice" in
*"Connect to Wi-Fi") do_connect ;;
*"Disconnect") do_disconnect ;;
*"Network Status") do_status ;;
*"VPN Switcher") do_vpn_switcher ;;
*"Cloudflare DNS"*) do_toggle_dns ;;
*"Toggle Wi-Fi") do_toggle_wifi ;;
*"Toggle Ethernet") do_toggle_ethernet ;;
*) exit 0 ;;
esac
