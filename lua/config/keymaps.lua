local map = vim.keymap.set

-- Jumplist (silent! suppresses stale mark errors)
map("n", "<C-o>", function() vim.cmd([[silent! exe "normal! \<C-o>"]]) end)
map("n", "<C-p>", function()
  local key = vim.api.nvim_replace_termcodes("<C-i>", true, false, true)
  vim.api.nvim_feedkeys(key, "nx", false)
end)

-- Window navigation
map("n", "<C-h>", "<C-w>h")
map("n", "<C-j>", "<C-w>j")
map("n", "<C-k>", "<C-w>k")
map("n", "<C-l>", "<C-w>l")
map("n", "<Tab>", "<C-w>w", { desc = "Cycle splits" })

-- Yank to system clipboard
map({ "n", "v" }, "<leader>y", '"+y', { desc = "Yank to clipboard" })
map("n", "<leader>Y", '"+y$', { desc = "Yank line to clipboard" })

-- Terminal: Escape exits to normal mode (split nav via Tab/arrows)
map("t", "<Esc>", "<C-\\><C-n>")

-- Arrow keys move between splits
map("n", "<Up>", "<C-W>k", { silent = true, desc = "Window up" })
map("n", "<Down>", "<C-W>j", { silent = true, desc = "Window down" })
map("n", "<Left>", "<C-W>h", { silent = true, desc = "Window left" })
map("n", "<Right>", "<C-W>l", { silent = true, desc = "Window right" })

-- Shift+Enter in terminal mode (newline without submit in Claude)
map("t", "<S-CR>", "\x1b[13;2u")

-- Neovide: Cmd+V paste
if vim.g.neovide then
  map({ "n", "v", "i", "t", "c" }, "<D-v>", function()
    local reg = vim.fn.getreg("+")
    vim.api.nvim_paste(reg, true, -1)
  end)
end

-- Tabs
for i = 1, 9 do
  map("n", "<leader>" .. i, i .. "gt", { desc = "Tab " .. i })
end
map("n", "<leader>0", ":tabnew<CR>:setlocal nobuflisted<CR>", { silent = true, desc = "New tab" })

-- Resume last telescope picker
map("n", "-", "<cmd>Telescope resume<cr>", { desc = "Resume last picker" })

-- Buffers (show markdown titles instead of filenames)
map("n", "<leader>b", function()
  local entry_display = require("telescope.pickers.entry_display")
  local displayer = entry_display.create({
    separator = " ",
    items = { { width = 4 }, { remaining = true } },
  })

  require("telescope.builtin").buffers({
    sort_mru = true,
    entry_maker = function(entry)
      local bufnr = entry.bufnr or entry
      local name = vim.api.nvim_buf_get_name(bufnr)
      local display_name = vim.fn.fnamemodify(name, ":t")

      -- For loaded markdown files, use the first heading as title
      if display_name:match("%.md$") and vim.api.nvim_buf_is_loaded(bufnr) then
        local first = vim.api.nvim_buf_get_lines(bufnr, 0, 1, false)[1] or ""
        local title = first:match("^#%s+(.+)")
        if title then display_name = title end
      end

      local indicator = (bufnr == vim.api.nvim_get_current_buf()) and "%" or " "
      return {
        value = entry,
        ordinal = display_name,
        display = function()
          return displayer({ { indicator, "TelescopeResultsComment" }, display_name })
        end,
        bufnr = bufnr,
        filename = name,
        lnum = entry.lnum or 1,
      }
    end,
  })
end, { desc = "Buffers" })

-- Diagnostics
map("n", "E", vim.diagnostic.open_float, { desc = "Show diagnostic" })

-- Lazygit
map("n", "<leader>lg", function()
  local buf = vim.api.nvim_create_buf(false, true)
  local win = vim.api.nvim_open_win(buf, true, {
    relative = "editor",
    row = 0,
    col = 0,
    width = vim.o.columns,
    height = vim.o.lines - 1,
    style = "minimal",
    zindex = 100,
  })
  vim.fn.termopen("lazygit", {
    on_exit = function()
      vim.schedule(function()
        if vim.api.nvim_win_is_valid(win) then vim.api.nvim_win_close(win, true) end
        if vim.api.nvim_buf_is_valid(buf) then vim.api.nvim_buf_delete(buf, { force = true }) end
      end)
    end,
  })
  vim.cmd("startinsert")
end)
