local M = {}
local headings = require("jk.headings")
local cache = {} -- path -> { backlinks, links, backlink_notes }
local ns = vim.api.nvim_create_namespace("zk_links")
local track_ns = vim.api.nvim_create_namespace("zk_track")
-- Track which buffers have appended lines via extmark
local buf_state = {} -- bufnr -> { mark_id = N, note_map = { [offset] = { path, heading_raw } } }

--- Fallback: find our tracking extmark even when buf_state is nil.
--- Extmarks in track_ns persist on the buffer independently of buf_state.
local function defensive_clear(bufnr)
  local marks = vim.api.nvim_buf_get_extmarks(bufnr, track_ns, 0, -1, {})
  if #marks == 0 then return false end

  local start_line = marks[1][2] -- 0-indexed line of first extmark
  local was_modifiable = vim.bo[bufnr].modifiable
  local was_modified = vim.bo[bufnr].modified
  vim.bo[bufnr].modifiable = true
  vim.api.nvim_buf_set_lines(bufnr, start_line, -1, false, {})
  vim.bo[bufnr].modifiable = was_modifiable
  vim.bo[bufnr].modified = was_modified
  vim.api.nvim_buf_clear_namespace(bufnr, track_ns, 0, -1)
  return true
end

local function clear_appended(bufnr)
  local state = buf_state[bufnr]
  if not state then return false end

  -- Read extmark's current position
  local pos = vim.api.nvim_buf_get_extmark_by_id(bufnr, track_ns, state.mark_id, {})
  if not pos or #pos == 0 then
    buf_state[bufnr] = nil
    return false
  end

  local start_line = pos[1] -- 0-indexed line
  local was_modifiable = vim.bo[bufnr].modifiable
  local was_modified = vim.bo[bufnr].modified
  vim.bo[bufnr].modifiable = true
  vim.api.nvim_buf_set_lines(bufnr, start_line, -1, false, {})
  vim.bo[bufnr].modifiable = was_modifiable
  vim.bo[bufnr].modified = was_modified

  buf_state[bufnr] = nil
  return true
end

local function format_entry(title, section)
  return "- " .. headings.label(title, section)
end

local function render(bufnr, path)
  clear_appended(bufnr)
  vim.api.nvim_buf_clear_namespace(bufnr, ns, 0, -1)
  vim.api.nvim_buf_clear_namespace(bufnr, track_ns, 0, -1)

  local c = cache[path]
  if not c then return end
  if not c.backlink_notes or #c.backlink_notes == 0 then return end

  local lines = { "", "Backlinks" }
  local note_map = {} -- offset from extmark -> { path, heading_raw }

  for _, note in ipairs(c.backlink_notes) do
    table.insert(lines, format_entry(note.title, note.section))
  end

  local start_line = vim.api.nvim_buf_line_count(bufnr)
  local was_modifiable = vim.bo[bufnr].modifiable
  local was_modified = vim.bo[bufnr].modified
  vim.bo[bufnr].modifiable = true
  vim.api.nvim_buf_set_lines(bufnr, start_line, -1, false, lines)
  vim.bo[bufnr].modifiable = was_modifiable
  vim.bo[bufnr].modified = was_modified

  -- Place tracking extmark at the first appended line (the blank line)
  local mark_id = vim.api.nvim_buf_set_extmark(bufnr, track_ns, start_line, 0, {})

  -- Build note_map with relative offsets and apply highlights
  for i, text in ipairs(lines) do
    local line_idx = start_line + i - 1
    local offset = i - 1 -- offset from extmark position

    if text:match("^- ") then
      for _, note in ipairs(c.backlink_notes) do
        if text == format_entry(note.title, note.section) then
          note_map[offset] = { path = note.path, heading_raw = note.heading_raw }
          break
        end
      end
      vim.api.nvim_buf_set_extmark(bufnr, ns, line_idx, 0, {
        line_hl_group = "ZkTag",
      })
    elseif text == "Backlinks" then
      vim.api.nvim_buf_set_extmark(bufnr, ns, line_idx, 0, {
        line_hl_group = "Comment",
      })
    end
  end

  buf_state[bufnr] = { mark_id = mark_id, note_map = note_map }
end

function M.refresh()
  local bufnr = vim.api.nvim_get_current_buf()
  local path = vim.api.nvim_buf_get_name(bufnr)
  if path == "" or vim.bo[bufnr].filetype ~= "markdown" then return end

  local zk_api = require("zk.api")
  local notebook = vim.env.ZK_NOTEBOOK_DIR
  local current_filename = vim.fn.fnamemodify(path, ":t:r")

  -- Count outbound links (for statusline only)
  zk_api.list(notebook, { linkedBy = { path }, select = { "path" } }, function(err, notes)
    cache[path] = cache[path] or {}
    if not err and notes then
      cache[path].links = #notes
    end
  end)

  -- Backlinks: find which section of the REMOTE note the link originates from
  zk_api.list(notebook, { linkTo = { path }, select = { "title", "path", "rawContent" } }, function(err, notes)
    cache[path] = cache[path] or {}
    if not err and notes then
      cache[path].backlinks = #notes
      local entries = {}
      for _, n in ipairs(notes) do
        local title = n.title or vim.fn.fnamemodify(n.path, ":t:r")
        local section, heading_raw = nil, nil
        if n.rawContent then
          section, heading_raw = headings.section_for(n.rawContent, current_filename)
        end
        entries[#entries + 1] = {
          title = title,
          path = n.path,
          section = section,
          heading_raw = heading_raw,
        }
      end
      table.sort(entries, function(a, b)
        local a_journal = a.path:match("%d%d%d%d%-%d%d%-%d%d%.md$") and true or false
        local b_journal = b.path:match("%d%d%d%d%-%d%d%-%d%d%.md$") and true or false
        if a_journal ~= b_journal then return not a_journal end
        if a_journal then return a.path < b.path end -- oldest first
        return a.title:lower() < b.title:lower()
      end)
      cache[path].backlink_notes = entries
    end
    vim.schedule(function()
      if vim.api.nvim_buf_is_valid(bufnr) then
        render(bufnr, path)
      end
    end)
  end)
end

--- Returns { path, heading_raw } if cursor is on an appended backlink line
function M.note_at_cursor()
  local bufnr = vim.api.nvim_get_current_buf()
  local state = buf_state[bufnr]
  if not state then return nil end

  -- Get extmark's current position
  local pos = vim.api.nvim_buf_get_extmark_by_id(bufnr, track_ns, state.mark_id, {})
  if not pos or #pos == 0 then return nil end

  local mark_line = pos[1] -- 0-indexed
  local cursor_line = vim.api.nvim_win_get_cursor(0)[1] - 1 -- 0-indexed
  local offset = cursor_line - mark_line
  return state.note_map[offset]
end

function M.backlinks()
  local path = vim.api.nvim_buf_get_name(0)
  local c = cache[path]
  if not c or not c.backlinks or c.backlinks == 0 then return "" end
  return c.backlinks .. "←"
end

function M.links()
  local path = vim.api.nvim_buf_get_name(0)
  local c = cache[path]
  if not c or not c.links or c.links == 0 then return "" end
  return c.links .. "→"
end

-- Strip appended lines before writing to disk or quitting
vim.api.nvim_create_autocmd("BufWritePre", {
  pattern = "*.md",
  callback = function(ev)
    clear_appended(ev.buf)
    defensive_clear(ev.buf)
  end,
})

vim.api.nvim_create_autocmd({ "BufEnter", "BufWritePost" }, {
  pattern = "*.md",
  callback = M.refresh,
})

-- Clean up buf_state for deleted buffers
vim.api.nvim_create_autocmd("BufDelete", {
  pattern = "*.md",
  callback = function(ev)
    buf_state[ev.buf] = nil
  end,
})

return M
