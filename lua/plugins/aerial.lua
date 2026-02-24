return {
  "stevearc/aerial.nvim",
  lazy = false,
  opts = {
    backends = { "treesitter" },
    layout = {
      default_direction = "float",
      max_width = 60,
    },
    float = {
      relative = "win",
      max_height = 0.7,
      max_width = 0.7,
      min_height = 0,
      min_width = 0,
    },
    autojump = true,
    filter_kind = false,
    icons = {
      Interface = "#",
    },
    show_guides = true,
    guides = {
      mid_item = "├ ",
      last_item = "└ ",
      nested_top = "│ ",
    },
  },
}
