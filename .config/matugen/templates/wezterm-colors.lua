-- Auto-generated Material You color palette for WezTerm by Matugen
return {
  foreground = '#{{ colors.on_surface.default.hex | replace: "#", "" }}',
  background = '#{{ colors.background.default.hex | replace: "#", "" }}',

  cursor_bg = '#{{ colors.primary.default.hex | replace: "#", "" }}',
  cursor_fg = '#{{ colors.on_primary.default.hex | replace: "#", "" }}',
  cursor_border = '#{{ colors.primary.default.hex | replace: "#", "" }}',

  selection_fg = '#{{ colors.on_secondary_container.default.hex | replace: "#", "" }}',
  selection_bg = '#{{ colors.secondary_container.default.hex | replace: "#", "" }}',

  scrollbar_thumb = '#{{ colors.surface_container.default.hex | replace: "#", "" }}',
  split = '#{{ colors.outline.default.hex | replace: "#", "" }}',

  ansi = {
    '#{{ colors.background.default.hex | replace: "#", "" }}',
    '#F7768E',
    '#9ECE6A',
    '#E0AF68',
    '#7AA2F7',
    '#{{ colors.primary.default.hex | replace: "#", "" }}',
    '#7DCFFF',
    '#{{ colors.on_surface.default.hex | replace: "#", "" }}',
  },
  brights = {
    '#{{ colors.outline.default.hex | replace: "#", "" }}',
    '#FF7A93',
    '#B9F27C',
    '#FF9E64',
    '#89B4FA',
    '#{{ colors.tertiary.default.hex | replace: "#", "" }}',
    '#B4F9F8',
    '#FFFFFF',
  },

  tab_bar = {
    background = '#{{ colors.background.default.hex | replace: "#", "" }}',
    active_tab = {
      bg_color = '#{{ colors.primary.default.hex | replace: "#", "" }}',
      fg_color = '#{{ colors.on_primary.default.hex | replace: "#", "" }}',
    },
    inactive_tab = {
      bg_color = '#{{ colors.surface_container.default.hex | replace: "#", "" }}',
      fg_color = '#{{ colors.on_surface_variant.default.hex | replace: "#", "" }}',
    },
    inactive_tab_hover = {
      bg_color = '#{{ colors.surface_container_highest.default.hex | replace: "#", "" }}',
      fg_color = '#{{ colors.on_surface.default.hex | replace: "#", "" }}',
    },
    new_tab = {
      bg_color = '#{{ colors.surface_container.default.hex | replace: "#", "" }}',
      fg_color = '#{{ colors.on_surface_variant.default.hex | replace: "#", "" }}',
    },
    new_tab_hover = {
      bg_color = '#{{ colors.primary.default.hex | replace: "#", "" }}',
      fg_color = '#{{ colors.on_primary.default.hex | replace: "#", "" }}',
    },
  },
}
