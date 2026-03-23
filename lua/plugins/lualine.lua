return {
  "nvim-lualine/lualine.nvim",
  dependencies = { "nvim-tree/nvim-web-devicons" },
  event = "VeryLazy",
  opts = {
    options = {
      component_separators = { left = "•", right = "•" },
      section_separators = { left = "", right = "" },
      globalstatus = true,
      disabled_filetypes = { statusline = { "qf" } },
      refresh = { statusline = 100 },
    },
    sections = {
      lualine_a = { "mode" },
      lualine_b = {
        function()
          if vim.bo.filetype ~= "markdown" then return "" end
          local title_line = vim.api.nvim_buf_get_lines(0, 0, 1, false)[1] or ""
          local title = title_line:match("^#%s+(.+)") or ""
          local row = vim.api.nvim_win_get_cursor(0)[1]
          for i = row, 1, -1 do
            local line = vim.api.nvim_buf_get_lines(0, i - 1, i, false)[1] or ""
            local h2 = line:match("^##%s+(.+)")
            if h2 then return title .. " > " .. h2 end
          end
          return title
        end,
      },
      lualine_c = {
        function()
          if vim.bo.filetype ~= "markdown" then return "" end
          local line = vim.api.nvim_buf_get_lines(0, 1, 2, false)[1] or ""
          return line:match("^tags:%s*(.+)") or ""
        end,
        function() return require("jk.zk_status").backlinks() end,
        function() return require("jk.zk_status").links() end,
      },
      lualine_x = {
        "diff",
      },
      lualine_y = {
        function()
          if vim.bo.filetype ~= "markdown" then return "" end
          local wc = vim.fn.wordcount()
          return wc.words .. " words"
        end,
      },
      lualine_z = { "progress" },
    },
  },
}
