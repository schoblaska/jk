local small_words = { "a", "an", "and", "as", "at", "but", "by", "for", "in", "nor", "of", "on", "or", "so", "the", "to", "up", "yet" }
local skip = {}
for _, w in ipairs(small_words) do skip[w] = true end

local function title_case(str)
  local words = {}
  for word in str:gmatch("%S+") do
    if #words == 0 or not skip[word:lower()] then
      table.insert(words, word:sub(1, 1):upper() .. word:sub(2):lower())
    else
      table.insert(words, word:lower())
    end
  end
  return table.concat(words, " ")
end

local function tag_under_cursor()
  local col = vim.api.nvim_win_get_cursor(0)[2]
  local line = vim.api.nvim_get_current_line()
  local start = 1
  while start <= #line do
    local s, e, tag = line:find("#([%w][%w_-]*)", start)
    if not s then break end
    if col >= s - 1 and col < e then return tag end
    start = e + 1
  end
end

local function follow_link(open_cmd)
  -- Close any preview/hover floats
  for _, win in ipairs(vim.api.nvim_list_wins()) do
    if vim.api.nvim_win_get_config(win).relative ~= "" then
      vim.api.nvim_win_close(win, true)
    end
  end
  local entry = require("jk.zk_status").note_at_cursor()
  if entry then
    local current_stem = vim.fn.expand("%:t:r")
    local notebook = vim.env.ZK_NOTEBOOK_DIR or ""
    vim.cmd[open_cmd](notebook .. "/" .. entry.path)
    vim.fn.search(vim.fn.escape(current_stem, "\\[].*^$~"), "cw")
    return
  end
  local tag = tag_under_cursor()
  if tag then
    require("jk.tag_picker")({ tag })
    return
  end
  local col = vim.api.nvim_win_get_cursor(0)[2]
  local line = vim.api.nvim_get_current_line()
  local start = 1
  while start <= #line do
    local s, e, link_text, target = line:find("%[([^%]]+)%]%(([^%)]+)%)", start)
    if not s then break end
    if col >= s - 1 and col < e then
      if target:match("^https?://") then
        vim.ui.open(target)
        return
      end
      local file_target = target:gsub("#.*$", "")
      if file_target ~= "" then
        local notebook = vim.env.ZK_NOTEBOOK_DIR or vim.fn.getcwd()
        local current_dir = vim.fn.expand("%:h")
        local abs_target = vim.fn.resolve(current_dir .. "/" .. file_target)
        if not abs_target:match("%.%w+$") then
          abs_target = abs_target .. ".md"
        end
        if vim.fn.filereadable(abs_target) == 0 then
          local rel = abs_target:sub(#notebook + 2)
          local dir = vim.fn.fnamemodify(rel, ":h")
          local basename = vim.fn.fnamemodify(rel, ":t:r")
          local opts = {}
          if basename:match("^%d%d%d%d%-%d%d%-%d%d$") then
            opts.group = "journal"
            opts.date = basename
          else
            opts.title = title_case(link_text)
          end
          if dir ~= "." then opts.dir = dir end
          require("zk.commands").get("ZkNew")(opts)
          return
        end
        vim.cmd[open_cmd](abs_target)
        return
      end
      break
    end
    start = e + 1
  end
  vim.lsp.buf.definition()
end

return {
  follow_link = follow_link,
  title_case = title_case,
  tag_under_cursor = tag_under_cursor,
}
