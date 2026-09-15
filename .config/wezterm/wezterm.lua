local wezterm = require 'wezterm'
local act = wezterm.action
local config = wezterm.config_builder()

-- ── Theme ─────────────────────────────────────────────────────────────────────
local ok, colors = pcall(require, 'colors')
if ok and colors then
  config.colors = colors
end

-- ── Font ──────────────────────────────────────────────────────────────────────
config.font = wezterm.font('JetBrainsMono Nerd Font')
config.font_size = 12.0

-- ── Cursor Trail ──────────────────────────────────────────────────────────────
config.default_cursor_style = 'SteadyBlock'
config.cursor_blink_rate = 0
config.cursor_trail = true
config.cursor_trail_decay = 0.30

-- ── Window ────────────────────────────────────────────────────────────────────
config.window_background_opacity = 0.82
config.window_decorations = 'NONE'
config.window_padding = { left = 14, right = 14, top = 14, bottom = 14 }
config.initial_cols = 100
config.initial_rows = 30
config.window_close_confirmation = 'NeverPrompt'
config.audible_bell = 'Disabled'
config.check_for_updates = false
config.scrollback_lines = 100000
config.exit_behavior = 'CloseOnCleanExit'

-- ── Shell ─────────────────────────────────────────────────────────────────────
config.default_prog = { '/usr/bin/zsh' }

-- ── Mouse & Links ─────────────────────────────────────────────────────────────
config.hide_mouse_cursor_when_typing = true
config.underline_position = -2

config.hyperlink_rules = wezterm.default_hyperlink_rules()
table.insert(config.hyperlink_rules, {
  regex = [[\b(?:file|ftp|ftps|gemini|git|gopher|http|https|irc|ircs|mailto|news|sftp|ssh):(?://)?\S+]],
  format = '$0',
})

-- Route all URL opens through rice-ctl
wezterm.on('open-uri', function(window, pane, uri)
  wezterm.background_child_process {
    '/home/pineapple/.local/bin/rice-ctl', 'open', uri,
  }
  return false
end)

config.mouse_bindings = {
  {
    event = { Up = { streak = 1, button = 'Left' } },
    mods = 'CTRL',
    action = act.OpenLinkAtMouseCursor,
  },
  {
    event = { Up = { streak = 1, button = 'Left' } },
    mods = 'SHIFT',
    action = act.OpenLinkAtMouseCursor,
  },
}

-- ── Tab Bar ───────────────────────────────────────────────────────────────────
config.enable_tab_bar = false

-- ── Leader (Ctrl+Space) ───────────────────────────────────────────────────────
config.leader = { key = 'Space', mods = 'CTRL', timeout_milliseconds = 1500 }

-- ── Keybindings ───────────────────────────────────────────────────────────────
local keys = {
  -- Clipboard
  { key = 'c', mods = 'CTRL|SHIFT', action = act.CopyTo 'Clipboard' },
  { key = 'v', mods = 'CTRL|SHIFT', action = act.PasteFrom 'Clipboard' },

  -- Tabs
  { key = 't', mods = 'SUPER',      action = act.SpawnTab 'CurrentPaneDomain' },
  { key = 't', mods = 'CTRL|SHIFT', action = act.SpawnTab 'CurrentPaneDomain' },
  { key = 'c', mods = 'LEADER',     action = act.SpawnTab 'CurrentPaneDomain' },
  { key = 'w', mods = 'CTRL',       action = act.CloseCurrentTab { confirm = false } },
  { key = 'w', mods = 'SUPER',      action = act.CloseCurrentTab { confirm = false } },
  { key = 'Tab', mods = 'CTRL',       action = act.ActivateTabRelative(1) },
  { key = 'Tab', mods = 'CTRL|SHIFT', action = act.ActivateTabRelative(-1) },

  -- Pane splits
  { key = '|', mods = 'LEADER',       action = act.SplitHorizontal { domain = 'CurrentPaneDomain' } },
  { key = '|', mods = 'LEADER|SHIFT', action = act.SplitHorizontal { domain = 'CurrentPaneDomain' } },
  { key = '%', mods = 'LEADER',       action = act.SplitHorizontal { domain = 'CurrentPaneDomain' } },
  { key = '%', mods = 'LEADER|SHIFT', action = act.SplitHorizontal { domain = 'CurrentPaneDomain' } },
  { key = '-', mods = 'LEADER',       action = act.SplitVertical { domain = 'CurrentPaneDomain' } },
  { key = '"', mods = 'LEADER',       action = act.SplitVertical { domain = 'CurrentPaneDomain' } },
  { key = '"', mods = 'LEADER|SHIFT', action = act.SplitVertical { domain = 'CurrentPaneDomain' } },

  -- Pane navigation
  { key = 'h', mods = 'LEADER', action = act.ActivatePaneDirection 'Left' },
  { key = 'j', mods = 'LEADER', action = act.ActivatePaneDirection 'Down' },
  { key = 'k', mods = 'LEADER', action = act.ActivatePaneDirection 'Up' },
  { key = 'l', mods = 'LEADER', action = act.ActivatePaneDirection 'Right' },

  -- Pane control
  { key = 'z', mods = 'LEADER', action = act.TogglePaneZoomState },
  { key = 'x', mods = 'LEADER', action = act.CloseCurrentPane { confirm = false } },

  -- Pane resize
  { key = 'LeftArrow',  mods = 'LEADER', action = act.AdjustPaneSize { 'Left', 5 } },
  { key = 'RightArrow', mods = 'LEADER', action = act.AdjustPaneSize { 'Right', 5 } },
  { key = 'UpArrow',    mods = 'LEADER', action = act.AdjustPaneSize { 'Up', 5 } },
  { key = 'DownArrow',  mods = 'LEADER', action = act.AdjustPaneSize { 'Down', 5 } },

  -- Copy mode
  { key = '[', mods = 'LEADER', action = act.ActivateCopyMode },

  -- URL / text picker
  {
    key = 'u', mods = 'LEADER',
    action = act.QuickSelectArgs {
      patterns = {
        [[\bhttps?://\S+]],
        [[\b(?:file|ftp|git)://\S+]],
        [[(?:[/\w.-]+/[/\w.-]+)]],
      },
    },
  },
  { key = 'e', mods = 'CTRL|SHIFT', action = act.QuickSelect },
  {
    key = 'p', mods = 'CTRL|SHIFT',
    action = act.QuickSelectArgs {
      patterns = {
        [[(?:[/\w.-]+/[/\w.-]+)]],
        [[\bhttps?://\S+]],
      },
    },
  },

  -- Scroll
  { key = 'PageUp',   action = act.ScrollByPage(-1) },
  { key = 'PageDown', action = act.ScrollByPage(1) },

  -- Font size
  { key = '=', mods = 'CTRL',       action = act.IncreaseFontSize },
  { key = '+', mods = 'CTRL',       action = act.IncreaseFontSize },
  { key = '+', mods = 'CTRL|SHIFT', action = act.IncreaseFontSize },
  { key = '-', mods = 'CTRL',       action = act.DecreaseFontSize },
  { key = '_', mods = 'CTRL|SHIFT', action = act.DecreaseFontSize },
  { key = '0', mods = 'CTRL',       action = act.ResetFontSize },
}

-- Leader+1..9 and Super+1..9 for direct tab switching
for i = 1, 9 do
  table.insert(keys, { key = tostring(i), mods = 'LEADER', action = act.ActivateTab(i - 1) })
  table.insert(keys, { key = tostring(i), mods = 'SUPER',  action = act.ActivateTab(i - 1) })
end

config.keys = keys

return config
