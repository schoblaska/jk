return {
  "kevinhwang91/nvim-hlslens",
  event = "CmdlineEnter",
  keys = {
    { "n", [[<Cmd>execute("normal! " . v:count1 . "n")<CR><Cmd>lua require("hlslens").start()<CR>]], desc = "Next match" },
    { "N", [[<Cmd>execute("normal! " . v:count1 . "N")<CR><Cmd>lua require("hlslens").start()<CR>]], desc = "Prev match" },
    { "*", [[*<Cmd>lua require("hlslens").start()<CR>]], desc = "Search word forward" },
    { "#", [[#<Cmd>lua require("hlslens").start()<CR>]], desc = "Search word backward" },
    {
      "<Esc>",
      function()
        vim.cmd("noh")
        for _, win in ipairs(vim.api.nvim_list_wins()) do
          if vim.api.nvim_win_get_config(win).relative ~= "" then
            pcall(vim.api.nvim_win_close, win, false)
          end
        end
      end,
      desc = "Clear search highlight and close floats",
    },
  },
  opts = {
    calm_down = true,
    nearest_float_when = "never",
  },
}
