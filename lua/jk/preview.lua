local function hover()
  local entry = require("jk.zk_status").note_at_cursor()
  if not entry then
    vim.lsp.buf.hover({ max_width = 80, max_height = 20 })
    return
  end
  local notebook = vim.env.ZK_NOTEBOOK_DIR or ""
  local abs = notebook .. "/" .. entry.path
  local lines = {}
  local f = io.open(abs, "r")
  if f then
    for line in f:lines() do lines[#lines + 1] = line end
    f:close()
  end
  if #lines == 0 then return end
  local buf = vim.api.nvim_create_buf(false, true)
  vim.api.nvim_buf_set_lines(buf, 0, -1, false, lines)
  vim.bo[buf].filetype = "markdown"
  local width = math.min(80, vim.o.columns - 4)
  local height = math.min(#lines, 20, math.floor(vim.o.lines * 0.6))
  local current_stem = vim.fn.expand("%:t:r")
  local scroll_to = 0
  for i, line in ipairs(lines) do
    if line:find(current_stem, 1, true) then
      scroll_to = i - 1
      break
    end
  end
  local cursor_row = vim.fn.screenrow()
  local screen_height = vim.o.lines
  local space_below = screen_height - cursor_row - 1
  local row
  if space_below >= height + 1 then
    row = 1
  else
    row = -height - 2
  end
  local win = vim.api.nvim_open_win(buf, false, {
    relative = "cursor",
    row = row,
    col = 1,
    width = width,
    height = height,
    style = "minimal",
    focusable = false,
  })
  if scroll_to > 0 then
    vim.api.nvim_buf_add_highlight(buf, -1, "TelescopePreviewLine", scroll_to, 0, -1)
    vim.api.nvim_win_set_cursor(win, { scroll_to + 1, 0 })
    vim.api.nvim_win_call(win, function() vim.cmd("normal! zt") end)
  end
  vim.api.nvim_create_autocmd({ "CursorMoved", "CursorMovedI", "BufLeave" }, {
    callback = function()
      if vim.api.nvim_win_is_valid(win) then
        vim.api.nvim_win_close(win, true)
      end
      return true
    end,
  })
end

return { hover = hover }
