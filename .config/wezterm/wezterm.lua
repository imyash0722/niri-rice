local wezterm = require 'wezterm'
local config = wezterm.config_builder()

-- ── Auto-Reload Watcher for Matugen Dynamic Theme ──
if wezterm.config_dir then
  wezterm.add_to_config_reload_watch_list(wezterm.config_dir .. '/colors.lua')
end

-- ── Typography (Exact Kitty Parity) ──
config.font = wezterm.font('JetBrainsMono Nerd Font')
config.font_size = 12.0

-- ── Shell & Process Behavior (Matching Kitty shell & close_on_child_death) ──
config.default_prog = { '/usr/bin/zsh' }
config.window_close_confirmation = 'NeverPrompt' -- confirm_os_window_close 0

-- ── Cursor: Solid Brick Rectangle with Continuous Tron Trail (No Translucency) ──
config.default_cursor_style = 'SteadyBlock'
config.force_reverse_video_cursor = false
config.cursor_blink_rate = 0
config.cursor_trail = true
config.cursor_trail_decay = 0.30
config.max_fps = 60

-- ── Window Styling & Translucency (Exact Kitty Parity with Frosted Glass Blur) ──
config.window_background_opacity = 0.90
config.text_background_opacity = 1.0

-- Compositor Background Blur (matching Kitty background_blur 1)
config.wayland_window_background_blur = true
config.macos_window_background_blur = 20
config.win32_system_backdrop = 'Acrylic'

config.window_padding = {
  left = 14,
  right = 14,
  top = 14,
  bottom = 14,
}
config.window_decorations = 'RESIZE'
config.enable_tab_bar = false

-- Window dimensions (matching Kitty initial_window_width 960, initial_window_height 600)
config.adjust_window_size_when_changing_font_size = false -- remember_window_size no
config.initial_cols = 100
config.initial_rows = 28

-- ── Mouse & Bell (Matching Kitty mouse_hide_wait, audio/visual bell) ──
config.hide_mouse_cursor_when_typing = true -- mouse_hide_wait 3.0
config.audible_bell = 'Disabled'           -- enable_audio_bell no
config.visual_bell = {
  fade_in_duration_ms = 0,
  fade_out_duration_ms = 0,
}

-- ── Dynamic Material You / Matugen Color Palette ──
local function load_matugen_colors()
  -- 1. Try native Matugen-generated colors.lua module
  local ok, matugen_colors = pcall(require, 'colors')
  if ok and type(matugen_colors) == 'table' then
    return matugen_colors
  end

  -- 2. Fallback to parsing Kitty's colors.conf directly if colors.lua is not yet loaded
  local home = os.getenv('HOME') or '/home/pineapple'
  local path = home .. '/.config/kitty/colors.conf'
  local f = io.open(path, 'r')
  if not f then return nil end

  local colors = {
    ansi = {},
    brights = {},
  }

  for line in f:lines() do
    local clean_line = line:gsub('#.*$', '')
    local key, val = clean_line:match('^%s*([%w_]+)%s+(#%x+)%s*$')
    if key and val then
      if key == 'foreground' then
        colors.foreground = val
      elseif key == 'background' then
        colors.background = val
      elseif key == 'cursor' then
        colors.cursor_bg = val
        colors.cursor_border = val
      elseif key == 'cursor_text_color' then
        colors.cursor_fg = val
      elseif key == 'selection_foreground' then
        colors.selection_fg = val
      elseif key == 'selection_background' then
        colors.selection_bg = val
      else
        local color_idx = key:match('^color(%d+)$')
        if color_idx then
          local idx = tonumber(color_idx)
          if idx >= 0 and idx <= 7 then
            colors.ansi[idx + 1] = val
          elseif idx >= 8 and idx <= 15 then
            colors.brights[idx - 7] = val
          end
        end
      end
    end
  end
  f:close()

  if #colors.ansi == 8 and #colors.brights == 8 then
    return colors
  end
  return nil
end

local theme_colors = load_matugen_colors()
if theme_colors then
  config.colors = theme_colors
end

-- ── Keybindings (Full Kitty Rice & Tmux Fast Switching Parity) ──
config.keys = {
  -- Tmux Fast Tab Switching parity with Kitty
  { key = 'Tab', mods = 'CTRL', action = wezterm.action.SendString '\x1b[27;5;9~' },
  { key = 'Tab', mods = 'CTRL|SHIFT', action = wezterm.action.SendString '\x1b[27;6;9~' },

  -- Clipboard copy & paste (ctrl+shift+c / ctrl+shift+v)
  { key = 'c', mods = 'CTRL|SHIFT', action = wezterm.action.CopyTo 'Clipboard' },
  { key = 'v', mods = 'CTRL|SHIFT', action = wezterm.action.PasteFrom 'Clipboard' },

  -- Font size adjustment (ctrl+plus, ctrl+minus, ctrl+0, plus keypad variants)
  { key = '=', mods = 'CTRL', action = wezterm.action.IncreaseFontSize },
  { key = '+', mods = 'CTRL|SHIFT', action = wezterm.action.IncreaseFontSize },
  { key = 'NumpadAdd', mods = 'CTRL', action = wezterm.action.IncreaseFontSize },
  { key = '-', mods = 'CTRL', action = wezterm.action.DecreaseFontSize },
  { key = '_', mods = 'CTRL|SHIFT', action = wezterm.action.DecreaseFontSize },
  { key = 'NumpadSubtract', mods = 'CTRL', action = wezterm.action.DecreaseFontSize },
  { key = '0', mods = 'CTRL', action = wezterm.action.ResetFontSize },
  { key = 'Numpad0', mods = 'CTRL', action = wezterm.action.ResetFontSize },

  -- Scrollback navigation (page_up / page_down)
  { key = 'PageUp', mods = 'NONE', action = wezterm.action.ScrollByPage(-1) },
  { key = 'PageDown', mods = 'NONE', action = wezterm.action.ScrollByPage(1) },

  -- Link / URL Hints (matching Kitty ctrl+shift+e open_url_with_hints)
  { key = 'e', mods = 'CTRL|SHIFT', action = wezterm.action.QuickSelect },
}

-- ── Mouse Bindings (Ctrl+Click to open URLs matching Kitty) ──
config.mouse_bindings = {
  {
    event = { Up = { streak = 1, button = 'Left' } },
    mods = 'CTRL',
    action = wezterm.action.OpenLinkAtMouseCursor,
  },
}

return config
