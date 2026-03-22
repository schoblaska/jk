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
    get_highlight = function(symbol, is_icon, is_collapsed)
      if is_icon then
        local level = math.min(symbol.level or 0, 5)
        return string.format("AerialLevel%dIcon", level)
      end
    end,
    keymaps = {
      ["<Up>"] = "actions.up_and_scroll",
      ["<Down>"] = "actions.down_and_scroll",
    },
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
