local wezterm = require 'wezterm'
local act = wezterm.action
local config = wezterm.config_builder()

-- ── 1. Dynamic Material You Theming via Matugen ──────────────────────────────
local ok, matugen_colors = pcall(require, 'colors')
if ok and matugen_colors then
  config.colors = matugen_colors
end

-- ── 2. Font & Typography (Translated from kitty.conf) ────────────────────────
config.font = wezterm.font('JetBrainsMono Nerd Font')
config.font_size = 12.0

-- ── 3. Custom Animated Cursor Trail (User's WezTerm Trail Engine) ───────────
config.default_cursor_style = 'SteadyBlock'
config.cursor_blink_rate = 0
config.cursor_trail = true
config.cursor_trail_decay = 0.30

-- ── 4. Window Styling & Translucency (Translated from kitty.conf) ───────────
config.window_background_opacity = 0.82
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
config.audible_bell = 'Disabled'
config.check_for_updates = false
config.scrollback_lines = 100000
config.exit_behavior = 'CloseOnCleanExit'

-- ── 5. Shell & Default Session ───────────────────────────────────────────────
config.default_prog = { '/usr/bin/zsh' }

-- ── 6. Mouse & Interaction (Translated from kitty.conf & tmux.conf) ─────────
config.hide_mouse_cursor_when_typing = true
config.underline_position = -2

config.hyperlink_rules = wezterm.default_hyperlink_rules()
table.insert(config.hyperlink_rules, {
  regex = [[\b(?:file|ftp|ftps|gemini|git|gopher|http|https|irc|ircs|kitty|mailto|news|sftp|ssh):(?://)?\S+]],
  format = '$0',
})

-- Universal URL Opener (Routes via rice-ctl open)
wezterm.on('open-uri', function(window, pane, uri)
  wezterm.background_child_process {
    '/home/pineapple/.local/bin/rice-ctl',
    'open',
    uri,
  }
  return false
end)

config.mouse_bindings = {
  -- Ctrl+Click to open links / URLs
  {
    event = { Up = { streak = 1, button = 'Left' } },
    mods = 'CTRL',
    action = act.OpenLinkAtMouseCursor,
  },
  -- Shift+Click fallback
  {
    event = { Up = { streak = 1, button = 'Left' } },
    mods = 'SHIFT',
    action = act.OpenLinkAtMouseCursor,
  },
}

-- ── 7. Tmux-Harmonized Tab Bar & Status Bar (Translated from tmux.conf) ─────
config.use_fancy_tab_bar = false
config.tab_bar_at_bottom = false
config.tab_max_width = 32
config.show_tab_index_in_tab_bar = false
config.hide_tab_bar_if_only_one_tab = true

wezterm.on('format-tab-title', function(tab, tabs, panes, cfg, hover, max_width)
  local title = tab.active_pane.title
  if #title > 24 then
    title = string.sub(title, 1, 21) .. '...'
  end
  local index = tab.tab_index + 1
  local colors = cfg.colors or {}
  local accent = colors.cursor_bg or '#adc6ff'
  local bg = colors.background or '#111318'
  local muted = colors.scrollbar_thumb or '#44474f'

  if tab.is_active then
    return {
      { Background = { Color = bg } },
      { Foreground = { Color = accent } },
      { Attribute = { Intensity = 'Bold' } },
      { Text = ' ' .. index .. ':' .. title .. '* ' },
    }
  else
    return {
      { Background = { Color = bg } },
      { Foreground = { Color = muted } },
      { Text = ' ' .. index .. ':' .. title .. ' ' },
    }
  end
end)

wezterm.on('update-right-status', function(window, pane)
  local time = wezterm.strftime '%H:%M '
  local colors = window:effective_config().colors or {}
  local accent = colors.cursor_bg or '#adc6ff'
  local fg = colors.foreground or '#e2e2e9'

  window:set_right_status(wezterm.format {
    { Foreground = { Color = fg } },
    { Text = time },
    { Foreground = { Color = accent } },
    { Text = '• ' },
  })
end)

-- ── 8. Multiplexer Leader Key (Prefix: Ctrl+Space from tmux.conf) ───────────
config.leader = { key = 'Space', mods = 'CTRL', timeout_milliseconds = 1500 }

-- ── 9. Keybindings (Full Migration of Kitty & Tmux) ──────────────────────────
local keys = {
  -- Clipboard (Kitty: map ctrl+shift+c / ctrl+shift+v)
  { key = 'c', mods = 'CTRL|SHIFT', action = act.CopyTo 'Clipboard' },
  { key = 'v', mods = 'CTRL|SHIFT', action = act.PasteFrom 'Clipboard' },

  -- Native Tab Creation (Super+T, Leader+c, Ctrl+Shift+T)
  { key = 't', mods = 'SUPER', action = act.SpawnTab 'CurrentPaneDomain' },
  { key = 't', mods = 'CTRL|SHIFT', action = act.SpawnTab 'CurrentPaneDomain' },
  { key = 'c', mods = 'LEADER', action = act.SpawnTab 'CurrentPaneDomain' },

  -- Close Tab / Window (Tmux: bind-key -n C-w kill-window)
  { key = 'w', mods = 'CTRL', action = act.CloseCurrentTab { confirm = false } },
  { key = 'w', mods = 'SUPER', action = act.CloseCurrentTab { confirm = false } },

  -- Fast Tab Switching (Kitty & Tmux: Ctrl+Tab and Ctrl+Shift+Tab)
  { key = 'Tab', mods = 'CTRL', action = act.ActivateTabRelative(1) },
  { key = 'Tab', mods = 'CTRL|SHIFT', action = act.ActivateTabRelative(-1) },

  -- Pane Splitting (Tmux: bind | split-window -h, bind - split-window -v)
  { key = '|', mods = 'LEADER', action = act.SplitHorizontal { domain = 'CurrentPaneDomain' } },
  { key = '|', mods = 'LEADER|SHIFT', action = act.SplitHorizontal { domain = 'CurrentPaneDomain' } },
  { key = '%', mods = 'LEADER', action = act.SplitHorizontal { domain = 'CurrentPaneDomain' } },
  { key = '%', mods = 'LEADER|SHIFT', action = act.SplitHorizontal { domain = 'CurrentPaneDomain' } },
  { key = '-', mods = 'LEADER', action = act.SplitVertical { domain = 'CurrentPaneDomain' } },
  { key = '"', mods = 'LEADER', action = act.SplitVertical { domain = 'CurrentPaneDomain' } },
  { key = '"', mods = 'LEADER|SHIFT', action = act.SplitVertical { domain = 'CurrentPaneDomain' } },

  -- Vim-Style Pane Navigation (Tmux: bind h/j/k/l select-pane)
  { key = 'h', mods = 'LEADER', action = act.ActivatePaneDirection 'Left' },
  { key = 'j', mods = 'LEADER', action = act.ActivatePaneDirection 'Down' },
  { key = 'k', mods = 'LEADER', action = act.ActivatePaneDirection 'Up' },
  { key = 'l', mods = 'LEADER', action = act.ActivatePaneDirection 'Right' },

  -- Pane Zoom Toggle (Tmux: bind z resize-pane -Z)
  { key = 'z', mods = 'LEADER', action = act.TogglePaneZoomState },

  -- Close Pane (Tmux: bind x kill-pane)
  { key = 'x', mods = 'LEADER', action = act.CloseCurrentPane { confirm = false } },

  -- Pane Resizing (Tmux: arrows with leader)
  { key = 'LeftArrow', mods = 'LEADER', action = act.AdjustPaneSize { 'Left', 5 } },
  { key = 'RightArrow', mods = 'LEADER', action = act.AdjustPaneSize { 'Right', 5 } },
  { key = 'UpArrow', mods = 'LEADER', action = act.AdjustPaneSize { 'Up', 5 } },
  { key = 'DownArrow', mods = 'LEADER', action = act.AdjustPaneSize { 'Down', 5 } },

  -- Vi Copy Mode (Tmux: bind-key -T copy-mode-vi / prefix + [)
  { key = '[', mods = 'LEADER', action = act.ActivateCopyMode },

  -- Fuzzy Link & Text Picker (Tmux: prefix + u link-picker, Kitty: ctrl+shift+e)
  {
    key = 'u',
    mods = 'LEADER',
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
    key = 'p',
    mods = 'CTRL|SHIFT',
    action = act.QuickSelectArgs {
      patterns = {
        [[(?:[/\w.-]+/[/\w.-]+)]],
        [[\bhttps?://\S+]],
      },
    },
  },

  -- Scroll (Kitty: map page_up / page_down)
  { key = 'PageUp', action = act.ScrollByPage(-1) },
  { key = 'PageDown', action = act.ScrollByPage(1) },

  -- Font Size & Zoom (Kitty: map ctrl+plus, ctrl+minus, ctrl+0)
  { key = '=', mods = 'CTRL', action = act.IncreaseFontSize },
  { key = '+', mods = 'CTRL', action = act.IncreaseFontSize },
  { key = '+', mods = 'CTRL|SHIFT', action = act.IncreaseFontSize },
  { key = '-', mods = 'CTRL', action = act.DecreaseFontSize },
  { key = '_', mods = 'CTRL|SHIFT', action = act.DecreaseFontSize },
  { key = '0', mods = 'CTRL', action = act.ResetFontSize },
}

-- Numeric Tab Switching (Tmux 1-based indexing: Leader+1..9 or Super+1..9)
for i = 1, 9 do
  table.insert(keys, {
    key = tostring(i),
    mods = 'LEADER',
    action = act.ActivateTab(i - 1),
  })
  table.insert(keys, {
    key = tostring(i),
    mods = 'SUPER',
    action = act.ActivateTab(i - 1),
  })
end

config.keys = keys

return config
