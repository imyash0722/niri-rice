return {
  "RedsXDD/neopywal.nvim",
  name = "neopywal",
  lazy = false,
  priority = 1000,
  opts = {
    transparent_background = true,
    file_watcher = true,
    custom_colors = function(C)
      local hsluv = require("neopywal.utils.hsluv")
      local function boost(hex, add_lightness, add_saturation)
        if not hex or hex == "" then return hex end
        local hsl = hsluv.hex_to_hsluv(hex)
        if add_saturation then
          hsl[2] = math.min(100, hsl[2] * (1 + add_saturation))
        end
        if add_lightness then
          local larpSpace = 100 - hsl[3]
          hsl[3] = hsl[3] + larpSpace * add_lightness
        end
        return hsluv.hsluv_to_hex(hsl)
      end

      local c1 = boost(C.color1, 0.45, 0.60)
      local c2 = boost(C.color2, 0.45, 0.60)
      local c3 = boost(C.color3, 0.45, 0.60)
      local c4 = boost(C.color4, 0.45, 0.60)
      local c5 = boost(C.color5, 0.45, 0.60)
      local c6 = boost(C.color6, 0.45, 0.60)

      return {
        all = {
          color1 = c1,
          color2 = c2,
          color3 = c3,
          color4 = c4,
          color5 = c5,
          color6 = c6,
          color9 = boost(C.color9 or C.color1, 0.50, 0.70),
          color10 = boost(C.color10 or C.color2, 0.50, 0.70),
          color11 = boost(C.color11 or C.color3, 0.50, 0.70),
          color12 = boost(C.color12 or C.color4, 0.50, 0.70),
          color13 = boost(C.color13 or C.color5, 0.50, 0.70),
          color14 = boost(C.color14 or C.color6, 0.50, 0.70),
          func = c2,
          keyword = c1,
          statement = c1,
          conditional = c1,
          operator = boost(C.color1, 0.40, 0.50),
          type = c6,
          structure = c6,
          constant = c3,
          variable = c4,
          identifier = boost(C.color4, 0.40, 0.50),
          number = c5,
          boolean = c5,
        },
      }
    end,
  },
}
