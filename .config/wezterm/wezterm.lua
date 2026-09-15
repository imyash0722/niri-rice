local wezterm = require 'wezterm'
local config = wezterm.config_builder()

-- ── 1. Dynamic Material You Theming (Translated from: include ./colors.conf) ──
local ok, matugen_colors = pcall(require, 'colors')
if ok and matugen_colors then
  config.colors = matugen_colors
end

-- ── 2. Font & Typography (Translated from: font_family, bold/italic, font_size 12.0) ──
config.font = wezterm.font('JetBrainsMono Nerd Font')
config.font_size = 12.0

-- ── 3. Cursor & Animation (Translated from: cursor_shape beam, cursor_trail 1) ──
-- Solid brick rectangle mode with Kitty-parity smooth continuous deformation trail
config.default_cursor_style = 'SteadyBlock'
config.cursor_blink_rate = 0
config.cursor_trail = true
config.cursor_trail_decay = 0.30

-- ── 4. Window Styling & Translucency (Translated from: background_opacity, window_padding_width, hide_window_decorations, etc.) ──
config.window_background_opacity = 0.90
config.window_decorations = 'NONE'
config.window_padding = {
  left = 14,
  right = 14,
  top = 14,
  bottom = 14,
}
config.initial_cols = 100
config.initial_rows = 30
config.window_close_confirmation = 'NeverPrompt'
config.hide_tab_bar_if_only_one_tab = true
config.use_fancy_tab_bar = false
config.tab_bar_at_bottom = false
config.audible_bell = 'Disabled'
config.check_for_updates = false

-- ── 5. Shell & Session (Translated from: shell /usr/bin/zsh, close_on_child_death no) ──
config.default_prog = { '/usr/bin/zsh' }
config.exit_behavior = 'CloseOnCleanExit'

-- ── 6. Mouse & Interaction (Translated from: mouse_hide_wait, detect_urls, underline_hyperlinks) ──
config.hide_mouse_cursor_when_typing = true
config.underline_position = -2

-- ── 7. URLs & Hyperlinks Clickability (Translated from: url_prefixes, open-actions.conf) ──
config.hyperlink_rules = wezterm.default_hyperlink_rules()
table.insert(config.hyperlink_rules, {
  regex = [[\b(?:file|ftp|ftps|gemini|git|gopher|http|https|irc|ircs|kitty|mailto|news|sftp|ssh):(?://)?\S+]],
  format = '$0',
})

-- Smart Opener: route clicked URLs through rice-ctl open (Matching open-actions.conf)
wezterm.on('open-uri', function(window, pane, uri)
  wezterm.background_child_process {
    '/home/pineapple/.local/bin/rice-ctl',
    'open',
    uri,
  }
  return false
end)

-- ── 8. Mouse Bindings (Translated from: mouse_map left/ctrl+left/shift+left) ──
config.mouse_bindings = {
  -- Ctrl+Click to open links / URLs
  {
    event = { Up = { streak = 1, button = 'Left' } },
    mods = 'CTRL',
    action = wezterm.action.OpenLinkAtMouseCursor,
  },
  -- Shift+Click fallback
  {
    event = { Up = { streak = 1, button = 'Left' } },
    mods = 'SHIFT',
    action = wezterm.action.OpenLinkAtMouseCursor,
  },
}

-- ── 9. Keybindings (Translated from: map in kitty.conf) ──
config.keys = {
  -- Clipboard & Copy/Paste (map ctrl+shift+c / ctrl+shift+v)
  { key = 'c', mods = 'CTRL|SHIFT', action = wezterm.action.CopyTo 'Clipboard' },
  { key = 'v', mods = 'CTRL|SHIFT', action = wezterm.action.PasteFrom 'Clipboard' },

  -- Tmux Fast Tab Switching (map ctrl+tab / ctrl+shift+tab)
  { key = 'Tab', mods = 'CTRL', action = wezterm.action.SendString '\x1b[27;5;9~' },
  { key = 'Tab', mods = 'CTRL|SHIFT', action = wezterm.action.SendString '\x1b[27;6;9~' },

  -- Scroll (map page_up / page_down)
  { key = 'PageUp', action = wezterm.action.ScrollByPage(-1) },
  { key = 'PageDown', action = wezterm.action.ScrollByPage(1) },

  -- Zoom / Font Scaling (map ctrl+plus, ctrl+equal, ctrl+minus, ctrl+underscore, ctrl+0)
  { key = '=', mods = 'CTRL', action = wezterm.action.IncreaseFontSize },
  { key = '+', mods = 'CTRL', action = wezterm.action.IncreaseFontSize },
  { key = '+', mods = 'CTRL|SHIFT', action = wezterm.action.IncreaseFontSize },
  { key = '-', mods = 'CTRL', action = wezterm.action.DecreaseFontSize },
  { key = '_', mods = 'CTRL|SHIFT', action = wezterm.action.DecreaseFontSize },
  { key = '0', mods = 'CTRL', action = wezterm.action.ResetFontSize },

  -- Keyboard Hints (map ctrl+shift+e open_url_with_hints & ctrl+shift+p hints)
  { key = 'e', mods = 'CTRL|SHIFT', action = wezterm.action.QuickSelect },
  {
    key = 'p',
    mods = 'CTRL|SHIFT',
    action = wezterm.action.QuickSelectArgs {
      patterns = {
        [[(?:[/\w.-]+/[/\w.-]+)]],
        [[\bhttps?://\S+]],
      },
    },
  },
}

return config
