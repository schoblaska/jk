return {
  "folke/flash.nvim",
  keys = {
    {
      "s",
      function()
        vim.o.cursorline = false
        require("flash").jump()
        vim.o.cursorline = true
      end,
      mode = { "n", "x", "o" },
      desc = "Flash",
    },
  },
  config = function(_, opts)
    require("flash").setup(opts)
    vim.api.nvim_set_hl(0, "FlashBackdrop", {})
    vim.api.nvim_set_hl(0, "FlashMatch", { link = "DiffChange" })
    vim.api.nvim_set_hl(0, "FlashCurrent", { link = "DiffChange" })
    vim.api.nvim_set_hl(0, "FlashLabel", { link = "@text.warning" })
  end,
  opts = {
    highlight = { backdrop = false },
    prompt = { enabled = false },
    modes = {
      search = { enabled = false },
      char = { jump_labels = true },
    },
  },
}
