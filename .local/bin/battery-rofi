#!/usr/bin/env python3
"""
ThinkBook Battery & Power Manager (Rofi Applet & CLI)
Features:
  - Percentage with visual gauge
  - Estimated time to discharge / charge
  - Battery health (energy_full vs energy_design) and cycle count
  - ThinkBook Conservation Mode switcher (0-60% battery health mode vs 0-100% lasting mode)
  - Platform power profile switcher
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
PLATFORM_PROFILE_PATH = "/sys/firmware/acpi/platform_profile"

def read_sysfs(path, default=""):
    if os.path.exists(path):
        try:
            with open(path, "r") as f:
                return f.read().strip()
        except Exception:
            pass
    return default

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
        notify("Battery Error", "ThinkBook conservation mode sysfs not found.", urgency="critical")
        return False
    
    val = str(1 if mode else 0)
    try:
        with open(p, "w") as f:
            f.write(val)
        return True
    except PermissionError:
        # Fallback to sudo if permissions somehow lost
        res = subprocess.run(["sudo", "sh", "-c", f"echo {val} > '{p}'"], capture_output=True)
        return res.returncode == 0
    except Exception as e:
        notify("Battery Error", f"Failed to set mode: {e}", urgency="critical")
        return False

def notify(title, message, icon="battery"):
    try:
        subprocess.run(["notify-send", "-a", "Battery Manager", "-i", icon, title, message], check=False)
    except Exception:
        pass

def get_battery_stats():
    capacity = int(read_sysfs(os.path.join(BAT_DIR, "capacity"), "0") or 0)
    status = read_sysfs(os.path.join(BAT_DIR, "status"), "Unknown")
    
    energy_now = int(read_sysfs(os.path.join(BAT_DIR, "energy_now"), "0") or 0)
    energy_full = int(read_sysfs(os.path.join(BAT_DIR, "energy_full"), "0") or 0)
    energy_design = int(read_sysfs(os.path.join(BAT_DIR, "energy_full_design"), "0") or 0)
    power_now = int(read_sysfs(os.path.join(BAT_DIR, "power_now"), "0") or 0)
    cycles = read_sysfs(os.path.join(BAT_DIR, "cycle_count"), "N/A")
    voltage_now = int(read_sysfs(os.path.join(BAT_DIR, "voltage_now"), "0") or 0)
    model = read_sysfs(os.path.join(BAT_DIR, "model_name"), "ThinkBook Battery")
    vendor = read_sysfs(os.path.join(BAT_DIR, "manufacturer"), "Lenovo")

    # Fallback to charge_* if energy_* not found
    if energy_now == 0 and energy_full == 0:
        charge_now = int(read_sysfs(os.path.join(BAT_DIR, "charge_now"), "0") or 0)
        charge_full = int(read_sysfs(os.path.join(BAT_DIR, "charge_full"), "0") or 0)
        charge_design = int(read_sysfs(os.path.join(BAT_DIR, "charge_full_design"), "0") or 0)
        current_now = int(read_sysfs(os.path.join(BAT_DIR, "current_now"), "0") or 0)
        energy_now = charge_now * (voltage_now / 1e6)
        energy_full = charge_full * (voltage_now / 1e6)
        energy_design = charge_design * (voltage_now / 1e6)
        power_now = current_now * (voltage_now / 1e6)

    health = (energy_full / energy_design * 100) if energy_design > 0 else 100.0
    energy_now_wh = energy_now / 1e6
    energy_full_wh = energy_full / 1e6
    energy_design_wh = energy_design / 1e6
    power_w = power_now / 1e6

    # Calculate time remaining
    time_str = "Calculating..."
    cons_mode = get_conservation_mode()

    if status == "Discharging":
        if power_now > 0:
            hrs = energy_now / power_now
            h = int(hrs)
            m = int((hrs - h) * 60)
            time_str = f"{h}h {m:02d}m remaining"
        else:
            time_str = "Discharging (Rate unmetered)"
    elif status == "Charging":
        target_energy = (0.60 * energy_full) if (cons_mode and capacity < 60) else energy_full
        diff_energy = max(0, target_energy - energy_now)
        if power_now > 0:
            hrs = diff_energy / power_now
            h = int(hrs)
            m = int((hrs - h) * 60)
            target_pct = "60%" if cons_mode else "100%"
            time_str = f"{h}h {m:02d}m until {target_pct}"
        else:
            time_str = "Charging"
    elif status in ("Full", "Not charging"):
        if cons_mode and 55 <= capacity <= 65:
            time_str = "Holding charge at ~60% (AC Powered)"
        elif capacity >= 95:
            time_str = "Fully Charged (AC Powered)"
        else:
            time_str = "Plugged In (AC Powered)"

    # Battery icon selection
    if status == "Charging":
        icon = "󰂄"
    elif capacity >= 95:
        icon = "󰁹"
    elif capacity >= 85:
        icon = "󰂂"
    elif capacity >= 75:
        icon = "󰂁"
    elif capacity >= 65:
        icon = "󰂀"
    elif capacity >= 55:
        icon = "󰁿"
    elif capacity >= 45:
        icon = "󰁾"
    elif capacity >= 35:
        icon = "󰁽"
    elif capacity >= 25:
        icon = "󰁼"
    elif capacity >= 15:
        icon = "󰁻"
    else:
        icon = "󰁺"

    # Gauge string
    filled = int(round(capacity / 10))
    gauge = "█" * filled + "░" * (10 - filled)

    profile = read_sysfs(PLATFORM_PROFILE_PATH, "balanced")

    return {
        "capacity": capacity,
        "status": status,
        "time_str": time_str,
        "health": health,
        "energy_now_wh": energy_now_wh,
        "energy_full_wh": energy_full_wh,
        "energy_design_wh": energy_design_wh,
        "power_w": power_w,
        "cycles": cycles,
        "voltage_v": voltage_now / 1e6,
        "model": model,
        "vendor": vendor,
        "cons_mode": cons_mode,
        "icon": icon,
        "gauge": gauge,
        "profile": profile,
    }

def print_status():
    s = get_battery_stats()
    mode_str = "0-60% Lifespan Mode (Conservation Active)" if s["cons_mode"] else "0-100% Lasting Mode (Full Capacity)"
    print(f"{s['icon']} Battery Level:     {s['capacity']}% [{s['gauge']}]")
    print(f"󱐋 Power State:       {s['status']} ({s['power_w']:.2f} W)")
    print(f"󰔐 Time Remaining:    {s['time_str']}")
    print(f" Battery Health:    {s['health']:.1f}% ({s['energy_full_wh']:.1f} / {s['energy_design_wh']:.1f} Wh · {s['cycles']} cycles)")
    print(f"󰚥 Charging Mode:     {mode_str}")
    print(f"󰈐 Platform Profile:  {s['profile'].capitalize()}")

def show_detailed_specs(s):
    details = (
        f"Model: {s['vendor']} {s['model']}\n"
        f"Health: {s['health']:.1f}%\n"
        f"Design Capacity: {s['energy_design_wh']:.2f} Wh\n"
        f"Full Capacity: {s['energy_full_wh']:.2f} Wh\n"
        f"Current Energy: {s['energy_now_wh']:.2f} Wh\n"
        f"Voltage: {s['voltage_v']:.2f} V\n"
        f"Cycle Count: {s['cycles']}\n"
        f"Power Draw: {s['power_w']:.2f} W\n"
        f"Conservation Mode: {'ON (60% limit)' if s['cons_mode'] else 'OFF (100% limit)'}"
    )
    notify(f"Battery Hardware Specs ({s['capacity']}%)", details, icon="battery")

def cycle_profile():
    curr = read_sysfs(PLATFORM_PROFILE_PATH, "balanced")
    mapping = {"performance": "balanced", "balanced": "low-power", "low-power": "performance"}
    nxt = mapping.get(curr, "balanced")
    try:
        with open(PLATFORM_PROFILE_PATH, "w") as f:
            f.write(nxt)
    except Exception:
        subprocess.run(["sudo", "sh", "-c", f"echo {nxt} > {PLATFORM_PROFILE_PATH}"], check=False)
    subprocess.run(["pkill", "-RTMIN+2", "waybar"], check=False)
    notify("Power Profile Changed", f"Profile set to: {nxt.capitalize()}", icon="power-profile")

def run_rofi_applet():
    while True:
        s = get_battery_stats()
        is_cons = s["cons_mode"] == 1

        if is_cons:
            mode_label = "󱐌 Mode:  0-60% Lifespan Mode (Conservation Active)"
            toggle_action = "󱤅 Switch to 0-100% Lasting Mode   (Full charge for travel)"
        else:
            mode_label = "󱤅 Mode:  0-100% Lasting Mode (Full Capacity Active)"
            toggle_action = "󱐌 Switch to 0-60% Lifespan Mode   (Caps at 60% for desk use)"

        power_info = f" ({s['power_w']:.1f}W)" if s['power_w'] > 0 else ""
        
        items = [
            f"{s['icon']} Level:   {s['capacity']}%  [{s['gauge']}]",
            f"󱐋 State:   {s['status']}{power_info}",
            f"󰔐 Time:    {s['time_str']}",
            f" Health:  {s['health']:.1f}%  ({s['energy_full_wh']:.1f}/{s['energy_design_wh']:.1f} Wh · {s['cycles']} cycles)",
            mode_label,
            "───────────────────────────────────────────────",
            f"󰚥 {toggle_action}",
            f"󰈐 Power Profile: [{s['profile'].capitalize()}] (Click to switch)",
            "󰛲 Detailed Specs & Hardware Info",
        ]

        menu_input = "\n".join(items)
        rofi_cmd = [
            "rofi",
            "-dmenu",
            "-i",
            "-p", "󰁹 Battery",
            "-theme-str",
            "window { width: 560px; } listview { lines: 9; }",
            "-format", "s"
        ]

        proc = subprocess.run(rofi_cmd, input=menu_input, text=True, capture_output=True)
        choice = proc.stdout.strip()

        if proc.returncode != 0 or not choice:
            break

        if "Switch to 0-60% Lifespan Mode" in choice:
            if set_conservation_mode(1):
                notify(
                    "ThinkBook Battery: Conservation Mode",
                    "Charging capped at 60% to protect battery health during desk use.",
                    icon="battery-charging"
                )
        elif "Switch to 0-100% Lasting Mode" in choice:
            if set_conservation_mode(0):
                notify(
                    "ThinkBook Battery: Full Capacity Mode",
                    "Charging unlocked up to 100% for maximum on-the-go runtime.",
                    icon="battery-full"
                )
        elif "Power Profile:" in choice:
            cycle_profile()
        elif "Detailed Specs" in choice or "Health:" in choice or "Level:" in choice:
            show_detailed_specs(s)
        elif "Refresh" in choice:
            continue
        else:
            # User clicked informational line, show specs
            show_detailed_specs(s)

def main():
    parser = argparse.ArgumentParser(description="ThinkBook Battery Manager & Rofi Applet")
    parser.add_argument("--status", action="store_true", help="Print battery status and exit")
    parser.add_argument("--toggle", action="store_true", help="Toggle between 0-60%% and 0-100%% modes")
    parser.add_argument("--conservation", action="store_true", help="Set 0-60%% conservation mode")
    parser.add_argument("--full", action="store_true", help="Set 0-100%% full capacity mode")

    args = parser.parse_args()

    if args.status:
        print_status()
    elif args.toggle:
        curr = get_conservation_mode()
        nxt = 0 if curr == 1 else 1
        set_conservation_mode(nxt)
        if nxt == 1:
            notify("ThinkBook Battery", "Switched to 0-60% Conservation Mode (Desk Lifespan)", icon="battery-charging")
            print("Conservation Mode: ENABLED (0-60% Lifespan)")
        else:
            notify("ThinkBook Battery", "Switched to 0-100% Full Capacity Mode (Travel Runtime)", icon="battery-full")
            print("Conservation Mode: DISABLED (0-100% Runtime)")
    elif args.conservation:
        set_conservation_mode(1)
        notify("ThinkBook Battery", "Conservation Mode Activated (60% Cap)", icon="battery-charging")
        print("Conservation Mode: ENABLED (0-60% Lifespan)")
    elif args.full:
        set_conservation_mode(0)
        notify("ThinkBook Battery", "Full Capacity Mode Activated (100% Cap)", icon="battery-full")
        print("Conservation Mode: DISABLED (0-100% Runtime)")
    else:
        run_rofi_applet()

if __name__ == "__main__":
    main()
