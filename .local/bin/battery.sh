#!/usr/bin/env python3
"""
ThinkBook Battery & Power Manager (Rofi Script Applet & CLI)
Supports:
  - In-place live updating (window does NOT close when toggling options)
  - ThinkBook Conservation Mode (0-60% lifespan vs 0-100% lasting mode)
  - ACPI platform profile switching (Performance / Balanced / Low-power)
  - Live battery stats, percentage gauge, time remaining, health, cycles
  - Fast CLI options (--status, --toggle, --conservation, --full)
"""

import os
import sys
import subprocess
import argparse

BAT_DIR = "/sys/class/power_supply/BAT0"
CONSERVATION_PATHS = [
    "/sys/bus/platform/drivers/ideapad_acpi/VPC2004:00/conservation_mode",
    "/sys/devices/pci0000:00/0000:00:14.3/PNP0C09:00/VPC2004:00/conservation_mode",
]
PLATFORM_PROFILE_PATHS = [
    "/sys/firmware/acpi/platform_profile",
    "/sys/class/platform-profile/platform-profile-0/profile",
]

def read_sysfs(path, default=""):
    if os.path.exists(path):
        try:
            with open(path, "r") as f:
                return f.read().strip()
        except Exception:
            pass
    return default

def write_sysfs(path, val):
    try:
        with open(path, "w") as f:
            f.write(str(val))
        return True
    except Exception:
        return False

def get_conservation_path():
    for p in CONSERVATION_PATHS:
        if os.path.exists(p):
            return p
    return None

def get_conservation_mode():
    p = get_conservation_path()
    if p:
        val = read_sysfs(p, "0")
        return int(val) if val.isdigit() else 0
    return 0

def set_conservation_mode(mode: int):
    p = get_conservation_path()
    if not p:
        notify("Battery Error", "Conservation mode sysfs not found.", icon="dialog-error")
        return False
    val = str(1 if mode else 0)
    if write_sysfs(p, val):
        return True
    res = subprocess.run(["sudo", "-n", "sh", "-c", f"echo {val} > '{p}'"], capture_output=True)
    return res.returncode == 0

def get_platform_profile():
    for p in PLATFORM_PROFILE_PATHS:
        val = read_sysfs(p)
        if val:
            return val
    return "balanced"

def set_platform_profile(profile: str):
    success = False
    for p in PLATFORM_PROFILE_PATHS:
        if os.path.exists(p):
            if write_sysfs(p, profile):
                success = True
                break
    if not success:
        for p in PLATFORM_PROFILE_PATHS:
            res = subprocess.run(["sudo", "-n", "sh", "-c", f"echo {profile} > '{p}'"], capture_output=True)
            if res.returncode == 0:
                success = True
                break
    if success:
        subprocess.run(["pkill", "-RTMIN+2", "waybar"], check=False)
    return success

def cycle_platform_profile():
    curr = get_platform_profile()
    choices = ["low-power", "balanced", "performance"]
    choices_str = read_sysfs("/sys/firmware/acpi/platform_profile_choices")
    if choices_str:
        avail = choices_str.split()
        if avail:
            choices = avail
    try:
        idx = choices.index(curr)
        nxt = choices[(idx + 1) % len(choices)]
    except ValueError:
        nxt = "balanced"

    if set_platform_profile(nxt):
        notify("Power Profile", f"Profile set to {nxt.capitalize()}", icon="power-profile")
    else:
        notify("Power Profile Error", "Failed to switch power profile", icon="dialog-error")

def notify(title, message, icon="battery"):
    icon_map = {
        "battery": "battery",
        "battery-full": "battery-full",
        "battery-charging": "battery-charging",
        "power-profile": "battery",
        "dialog-error": "dialog-error",
    }
    ic = icon_map.get(icon, "battery")
    if os.path.exists(f"/home/pineapple/.icons/icon_scripts/dunst/battery/{ic}.svg"):
        icon_path = f"/home/pineapple/.icons/icon_scripts/dunst/battery/{ic}.svg"
    else:
        icon_path = ic

    cmd = ["dunstify", "-a", "battery", "-i", icon_path, title, message]
    subprocess.run(cmd, check=False)

def get_battery_stats():
    status = read_sysfs(f"{BAT_DIR}/status", "Discharging")
    capacity = int(read_sysfs(f"{BAT_DIR}/capacity", "0"))
    power_now_uw = int(read_sysfs(f"{BAT_DIR}/power_now", "0"))
    energy_now_uwh = int(read_sysfs(f"{BAT_DIR}/energy_now", "0"))
    energy_full_uwh = int(read_sysfs(f"{BAT_DIR}/energy_full", "0"))
    energy_design_uwh = int(read_sysfs(f"{BAT_DIR}/energy_full_design", "0"))
    voltage_now_uv = int(read_sysfs(f"{BAT_DIR}/voltage_now", "0"))
    cycles = int(read_sysfs(f"{BAT_DIR}/cycle_count", "0"))
    model = read_sysfs(f"{BAT_DIR}/model_name", "Battery")
    vendor = read_sysfs(f"{BAT_DIR}/manufacturer", "Lenovo")
    cons_mode = get_conservation_mode()

    power_w = power_now_uw / 1_000_000.0
    energy_now_wh = energy_now_uwh / 1_000_000.0
    energy_full_wh = energy_full_uwh / 1_000_000.0
    energy_design_wh = energy_design_uwh / 1_000_000.0
    voltage_v = voltage_now_uv / 1_000_000.0

    health = (energy_full_wh / energy_design_wh * 100.0) if energy_design_wh > 0 else 100.0

    if status == "Discharging" and power_w > 0.5:
        hours_left = energy_now_wh / power_w
        hrs = int(hours_left)
        mins = int((hours_left - hrs) * 60)
        time_str = f"{hrs}h {mins}m left"
    elif status == "Charging" and power_w > 0.5:
        hours_to_full = (energy_full_wh - energy_now_wh) / power_w
        hrs = int(hours_to_full)
        mins = int((hours_to_full - hrs) * 60)
        time_str = f"{hrs}h {mins}m to full"
    elif status == "Full":
        time_str = "Full (AC)"
    else:
        if power_w <= 0.5 and status != "Discharging":
            time_str = "Full (AC)"
        else:
            time_str = "AC Powered"

    if status == "Charging":
        icon = "󰂄"
    elif capacity >= 95:
        icon = "󰁹"
    elif capacity >= 80:
        icon = "󰂂"
    elif capacity >= 60:
        icon = "󰂀"
    elif capacity >= 40:
        icon = "󰁾"
    elif capacity >= 20:
        icon = "󰁼"
    else:
        icon = "󰁺"

    filled = int(round(capacity / 10))
    gauge = "█" * filled + "░" * (10 - filled)
    profile = get_platform_profile()

    return {
        "capacity": capacity,
        "status": status,
        "power_w": power_w,
        "energy_now_wh": energy_now_wh,
        "energy_full_wh": energy_full_wh,
        "energy_design_wh": energy_design_wh,
        "voltage_v": voltage_v,
        "cycles": cycles,
        "health": health,
        "time_str": time_str,
        "model": model,
        "vendor": vendor,
        "cons_mode": cons_mode,
        "icon": icon,
        "gauge": gauge,
        "profile": profile,
    }

def print_status():
    s = get_battery_stats()
    mode_str = "60% Lifespan (Conservation)" if s["cons_mode"] else "100% Full Capacity"
    print(f"{s['icon']} Level: {s['capacity']}% [{s['gauge']}]")
    print(f"󱐋 State: {s['status']} ({s['power_w']:.2f}W)")
    print(f"󱑂 Time: {s['time_str']}")
    print(f" Health: {s['health']:.1f}% ({s['energy_full_wh']:.1f}/{s['energy_design_wh']:.1f} Wh · {s['cycles']}c)")
    print(f"󰚥 Mode: {mode_str}")
    print(f"󰈐 Profile: {s['profile'].capitalize()}")

def show_detailed_specs(s):
    details = (
        f"Model: {s['vendor']} {s['model']}\n"
        f"Health: {s['health']:.1f}%\n"
        f"Design: {s['energy_design_wh']:.2f} Wh\n"
        f"Full: {s['energy_full_wh']:.2f} Wh\n"
        f"Current: {s['energy_now_wh']:.2f} Wh\n"
        f"Voltage: {s['voltage_v']:.2f} V\n"
        f"Cycles: {s['cycles']}\n"
        f"Power Draw: {s['power_w']:.2f} W\n"
        f"Conservation: {'ON (60%)' if s['cons_mode'] else 'OFF (100%)'}\n"
        f"Profile: {s['profile'].capitalize()}"
    )
    notify(f"Specs ({s['capacity']}%)", details, icon="battery")

def render_script_mode_entries(s):
    is_cons = s["cons_mode"] == 1
    if is_cons:
        mode_label = "󰚥 Mode: 60% Desk Lifespan"
        toggle_action = "󱤅 Switch to 100% Full Capacity"
    else:
        mode_label = "󰚥 Mode: 100% Full Capacity"
        toggle_action = "󱐌 Switch to 60% Desk Lifespan"

    power_info = f" ({s['power_w']:.1f}W)" if s['power_w'] > 0 else ""

    print("\0prompt\x1f󰁹 Battery")
    print("\0keep-selection\x1ftrue")
    print("\0no-custom\x1ftrue")

    items = [
        f"{s['icon']} Level: {s['capacity']}% [{s['gauge']}]",
        f"󱐋 State: {s['status']}{power_info}",
        f"󱑂 Time: {s['time_str']}",
        f" Health: {s['health']:.1f}% ({s['energy_full_wh']:.1f}/{s['energy_design_wh']:.1f}Wh · {s['cycles']}c)",
        mode_label,
        f"󰈐 Profile: [{s['profile'].capitalize()}] (Cycle)",
        toggle_action,
        "󰛲 Specs & Details",
        "󰅖 Close",
    ]
    for item in items:
        print(item)

def handle_rofi_script_mode():
    retv = os.environ.get("ROFI_RETV", "0")
    arg = sys.argv[1] if len(sys.argv) > 1 else ""

    if retv != "0" and arg:
        if "Close" in arg:
            sys.exit(0)
        elif "Switch to 60%" in arg or "60% Desk" in arg:
            if set_conservation_mode(1):
                notify(
                    "ThinkBook Battery: Conservation Mode",
                    "Charging capped at 60% for desk battery health.",
                    icon="battery-charging"
                )
        elif "Switch to 100%" in arg or "100% Full" in arg:
            if set_conservation_mode(0):
                notify(
                    "ThinkBook Battery: Full Capacity Mode",
                    "Charging unlocked to 100% for maximum runtime.",
                    icon="battery-full"
                )
        elif "Profile:" in arg:
            cycle_platform_profile()
        elif "Specs" in arg or "Health:" in arg or "Level:" in arg:
            s = get_battery_stats()
            show_detailed_specs(s)

    s = get_battery_stats()
    render_script_mode_entries(s)

def launch_rofi_gui():
    script_path = os.path.abspath(__file__)
    cmd = [
        "rofi",
        "-show", "battery",
        "-modes", f"battery:{script_path}",
        "-theme-str", "window { width: 420px; location: center; anchor: center; } listview { lines: 9; } entry { enabled: false; }"
    ]
    subprocess.run(cmd)

def main():
    if "ROFI_RETV" in os.environ or (len(sys.argv) > 1 and not sys.argv[1].startswith("--")):
        handle_rofi_script_mode()
        return

    parser = argparse.ArgumentParser(description="ThinkBook Battery Manager & Rofi Applet")
    parser.add_argument("--status", action="store_true", help="Print battery status and exit")
    parser.add_argument("--toggle", action="store_true", help="Toggle between 0-60%% and 0-100%% modes")
    parser.add_argument("--conservation", action="store_true", help="Set 0-60%% conservation mode")
    parser.add_argument("--full", action="store_true", help="Set 0-100%% full capacity mode")
    parser.add_argument("--cycle-profile", action="store_true", help="Cycle ACPI power profile")

    args = parser.parse_args()

    if args.status:
        print_status()
    elif args.toggle:
        curr = get_conservation_mode()
        nxt = 0 if curr == 1 else 1
        set_conservation_mode(nxt)
        if nxt == 1:
            notify("ThinkBook Battery", "Switched to 60% Conservation Mode", icon="battery-charging")
            print("Conservation Mode: ENABLED (60%)")
        else:
            notify("ThinkBook Battery", "Switched to 100% Full Capacity Mode", icon="battery-full")
            print("Conservation Mode: DISABLED (100%)")
    elif args.conservation:
        set_conservation_mode(1)
        notify("ThinkBook Battery", "Conservation Mode (60% Cap)", icon="battery-charging")
        print("Conservation Mode: ENABLED (60%)")
    elif args.full:
        set_conservation_mode(0)
        notify("ThinkBook Battery", "Full Capacity Mode (100% Cap)", icon="battery-full")
        print("Conservation Mode: DISABLED (100%)")
    elif args.cycle_profile:
        cycle_platform_profile()
    else:
        launch_rofi_gui()

if __name__ == "__main__":
    main()
