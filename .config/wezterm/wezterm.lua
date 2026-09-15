local wezterm = require 'wezterm'
local config = wezterm.config_builder()

-- ── Dynamic Material You Theming via Matugen ──
local ok, matugen_colors = pcall(require, 'colors')
if ok and matugen_colors then
  config.colors = matugen_colors
end

-- ── Font & Typography ──
config.font = wezterm.font('JetBrainsMono Nerd Font')
config.font_size = 12.0

-- ── Cursor & Animation ──
config.default_cursor_style = 'SteadyBlock'
config.cursor_blink_rate = 0
config.cursor_trail = true
config.cursor_trail_decay = 0.30

-- ── Window Styling & Translucency ──
config.window_background_opacity = 0.90
config.window_decorations = 'NONE'
config.window_padding = {
  left = 14,
  right = 14,
  top = 14,
  bottom = 14,
}
config.window_close_confirmation = 'NeverPrompt'
config.hide_tab_bar_if_only_one_tab = true
config.use_fancy_tab_bar = false
config.tab_bar_at_bottom = false
config.audible_bell = 'Disabled'
config.check_for_updates = false

-- ── Shell ──
config.default_prog = { '/usr/bin/zsh' }

-- ── Keybindings (Matching Kitty Rice Shortcuts) ──
config.keys = {
  { key = 'c', mods = 'CTRL|SHIFT', action = wezterm.action.CopyTo 'Clipboard' },
  { key = 'v', mods = 'CTRL|SHIFT', action = wezterm.action.PasteFrom 'Clipboard' },
  { key = 'Tab', mods = 'CTRL', action = wezterm.action.SendString '\x1b[27;5;9~' },
  { key = 'Tab', mods = 'CTRL|SHIFT', action = wezterm.action.SendString '\x1b[27;6;9~' },
  { key = 'PageUp', action = wezterm.action.ScrollByPage(-1) },
  { key = 'PageDown', action = wezterm.action.ScrollByPage(1) },
  { key = '=', mods = 'CTRL', action = wezterm.action.IncreaseFontSize },
  { key = '+', mods = 'CTRL', action = wezterm.action.IncreaseFontSize },
  { key = '+', mods = 'CTRL|SHIFT', action = wezterm.action.IncreaseFontSize },
  { key = '-', mods = 'CTRL', action = wezterm.action.DecreaseFontSize },
  { key = '_', mods = 'CTRL|SHIFT', action = wezterm.action.DecreaseFontSize },
  { key = '0', mods = 'CTRL', action = wezterm.action.ResetFontSize },
  { key = 'e', mods = 'CTRL|SHIFT', action = wezterm.action.QuickSelect },
}

-- ── Mouse Bindings ──
config.mouse_bindings = {
  {
    event = { Up = { streak = 1, button = 'Left' } },
    mods = 'CTRL',
    action = wezterm.action.OpenLinkAtMouseCursor,
  },
}

return config
