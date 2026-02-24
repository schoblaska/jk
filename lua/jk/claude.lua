local M = {}

local function claude_width()
  local cols = vim.o.columns
  return math.floor(cols * (cols <= 80 and 0.5 or 0.3))
end

local function set_win_opts(w)
  vim.wo[w].number = false
  vim.wo[w].relativenumber = false
  vim.wo[w].signcolumn = "no"
  vim.wo[w].spell = false
  vim.wo[w].winhighlight = "Normal:NormalDark,CursorLine:CursorLineDark"
end

local function get_bufs()
  return vim.t.claude_bufs or {}
end

local function set_bufs(list)
  vim.t.claude_bufs = list
end

--- Remove dead buffers from this tab's list.
local function prune()
  local live = {}
  for _, b in ipairs(get_bufs()) do
    if vim.api.nvim_buf_is_valid(b) then
      table.insert(live, b)
    end
  end
  set_bufs(live)
end

--- Return list of windows in this tab showing a Claude buffer.
local function claude_wins()
  prune()
  local wins = {}
  local buf_set = {}
  for _, b in ipairs(get_bufs()) do buf_set[b] = true end
  for _, w in ipairs(vim.api.nvim_tabpage_list_wins(0)) do
    if buf_set[vim.api.nvim_win_get_buf(w)] then
      table.insert(wins, w)
    end
  end
  return wins
end

--- Open a new Claude terminal in the left sidebar column.
function M.open()
  local existing = claude_wins()

  if #existing > 0 then
    vim.api.nvim_set_current_win(existing[#existing])
    vim.cmd("belowright split")
  else
    vim.cmd("topleft vsplit")
    local w = vim.api.nvim_get_current_win()
    vim.api.nvim_win_set_width(w, claude_width())
  end

  vim.cmd("terminal claude --settings '{\"editorMode\":\"normal\"}'")
  local w = vim.api.nvim_get_current_win()
  set_win_opts(w)
  local b = vim.api.nvim_get_current_buf()
  vim.bo[b].bufhidden = "hide"
  local list = get_bufs()
  table.insert(list, b)
  set_bufs(list)
  vim.keymap.set("n", "q", "<cmd>bd!<cr>", { buffer = b, nowait = true })
  vim.keymap.set("n", "<Esc>", function()
    local chan = vim.bo[b].channel
    if chan and chan > 0 then
      vim.api.nvim_chan_send(chan, "\27")
    end
  end, { buffer = b, nowait = true })
  vim.cmd("startinsert")
end

--- Toggle all Claude windows: hide if any visible, restore most recent if none.
function M.toggle()
  local wins = claude_wins()

  if #wins > 0 then
    for _, w in ipairs(wins) do
      vim.api.nvim_win_close(w, true)
    end
    return
  end

  prune()
  local bufs = get_bufs()
  if #bufs > 0 then
    for i, b in ipairs(bufs) do
      if i == 1 then
        vim.cmd("topleft vsplit")
        local w = vim.api.nvim_get_current_win()
        vim.api.nvim_win_set_width(w, claude_width())
        set_win_opts(w)
        vim.api.nvim_win_set_buf(w, b)
      else
        vim.cmd("belowright split")
        local w = vim.api.nvim_get_current_win()
        set_win_opts(w)
        vim.api.nvim_win_set_buf(w, b)
      end
    end
    vim.cmd("startinsert")
    return
  end

  M.open()
end

return M
