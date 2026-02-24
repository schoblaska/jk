return {
  "folke/snacks.nvim",
  lazy = false,
  priority = 900,
  opts = {
    notifier = {
      enabled = true,
      timeout = 3000,
      top_down = true,
      style = "compact",
      icons = { info = "", warn = "", error = "" },
    },
  },
}
