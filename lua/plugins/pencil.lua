return {
  "preservim/vim-pencil",
  init = function()
    vim.g["pencil#wrapModeDefault"] = "soft"
    vim.g["pencil#conceallevel"] = 0
    vim.api.nvim_create_autocmd("FileType", {
      pattern = "markdown",
      callback = function()
        vim.cmd("PencilSoft")
        -- Override pencil's buffer-local arrow key remaps to keep split navigation
        local opts = { buffer = true, silent = true }
        vim.keymap.set("n", "<Up>", "<C-W>k", opts)
        vim.keymap.set("n", "<Down>", "<C-W>j", opts)
      end,
    })
  end,
}
