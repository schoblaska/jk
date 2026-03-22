local pickers = require("telescope.pickers")
local finders = require("telescope.finders")
local conf = require("telescope.config").values
local sorters = require("telescope.sorters")
local headings = require("jk.headings")

--- Build path→title map from zk list
local function title_map()
  local raw = vim.fn.system({ "zk", "list", "--quiet", "--format", "{{path}}\t{{title}}" })
  local map = {}
  for line in raw:gmatch("[^\n]+") do
    local path, title = line:match("^(.-)\t(.+)$")
    if path and title then map[path] = title end
  end
  return map
end

return function()
  local titles = title_map()
  headings.clear_cache()

  -- Find unchecked checkboxes: - [<anything except x/X>]
  local rg = vim.fn.systemlist({
    "rg", "--vimgrep", "--no-heading", "-e", "^\\s*- \\[[^xX]\\]", "--glob", "*.md",
  })

  -- Collect file modification times
  local mtimes = {}
  local function get_mtime(path)
    if mtimes[path] == nil then
      local stat = vim.uv.fs_stat(path)
      mtimes[path] = stat and stat.mtime.sec or 0
    end
    return mtimes[path]
  end

  local results = {}
  for _, line in ipairs(rg) do
    local file, lnum, col, text = line:match("^(.-):(%d+):(%d+):(.*)$")
    if file then
      lnum = tonumber(lnum)
      col = tonumber(col)
      text = text:gsub("^%s+", "")
      local title = titles[file] or file:match("([^/]+)%.md$") or file
      local heading = headings.at(file, lnum)
      results[#results + 1] = {
        file = file,
        lnum = lnum,
        col = col,
        text = text,
        title = title,
        heading = heading,
        mtime = get_mtime(file),
      }
    end
  end

  table.sort(results, function(a, b)
    if a.mtime ~= b.mtime then return a.mtime > b.mtime end
    if a.file ~= b.file then return a.file < b.file end
    return a.lnum < b.lnum
  end)

  pickers.new({}, {
    prompt_title = "TODOs",
    finder = finders.new_table({
      results = results,
      entry_maker = function(item)
        local label = headings.label(item.title, item.heading)
        local display = label .. ": " .. item.text
        return {
          value = item,
          display = display,
          ordinal = display,
          filename = item.file,
          lnum = item.lnum,
          col = item.col,
        }
      end,
    }),
    sorter = conf.generic_sorter({}),
    previewer = conf.grep_previewer({}),
  }):find()
end
