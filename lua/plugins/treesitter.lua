return {
  "nvim-treesitter/nvim-treesitter",
  branch = "master",
  build = ":TSUpdate",
  config = function()
    require("nvim-treesitter.configs").setup({
      ensure_installed = { "markdown", "markdown_inline", "lua", "ruby" },
      highlight = { enable = true },
    })
  end,
}
