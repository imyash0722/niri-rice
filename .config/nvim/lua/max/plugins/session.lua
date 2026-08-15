-- Lua
return {
  "folke/persistence.nvim",
  lazy = false,
  config = function()
    local persistence = require("persistence")
    persistence.setup({
      need = 1,
      branch = true,
    })

    -- Only auto-restore if the session was saved within the last 24 hours
    local session_file = persistence.current()
    if session_file then
      local stat = (vim.uv or vim.loop).fs_stat(session_file)
      if stat and stat.mtime then
        local diff_seconds = os.time() - stat.mtime.sec
        if diff_seconds < (24 * 60 * 60) then
          persistence.load()
        end
      end
    end
  end
}
