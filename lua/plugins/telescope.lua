return {
  "nvim-telescope/telescope.nvim",
  dependencies = {
    "nvim-lua/plenary.nvim",
    { "nvim-telescope/telescope-fzf-native.nvim", build = "make" },
  },
  config = function()
    local telescope = require("telescope")

    local actions = require("telescope.actions")

    telescope.setup({
      defaults = {
        mappings = {
          i = {
            ["<C-q>"] = actions.send_selected_to_qflist + actions.open_qflist,
            ["<C-s>"] = actions.select_horizontal,
          },
          n = {
            ["<C-q>"] = actions.send_selected_to_qflist + actions.open_qflist,
            ["<C-s>"] = actions.select_horizontal,
          },
        },
        previewer = true,
        preview = { treesitter = true },
        prompt_prefix = " ",
        selection_caret = " ",
        sorting_strategy = "ascending",
        layout_strategy = "flex",
        preview_title = false,
        dynamic_preview_title = true,
        borderchars = { " ", " ", " ", " ", " ", " ", " ", " " },
        layout_config = {
          horizontal = {
            prompt_position = "top",
            preview_width = 0.5,
            preview_cutoff = 90,
          },
          vertical = {
            prompt_position = "top",
            preview_height = 0.60,
            preview_cutoff = 10,
          },
          flex = { flip_columns = 120 },
          anchor = "CENTER",
          width = { 0.92, max = 140 },
          height = { 0.90, max = 50 },
        },
      },
    })

    telescope.load_extension("fzf")

    vim.api.nvim_create_autocmd("User", {
      pattern = "TelescopePreviewerLoaded",
      callback = function()
        vim.wo.wrap = true
        vim.wo.linebreak = true
        vim.fn.matchadd("ZkTag", "\\#[[:alnum:]][[:alnum:]_-]*")
      end,
    })
  end,
}
