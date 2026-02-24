local M = {}

local pickers = require("telescope.pickers")
local finders = require("telescope.finders")
local sorters = require("telescope.sorters")
local conf = require("telescope.config").values
local actions = require("telescope.actions")
local action_state = require("telescope.actions.state")

local headings = require("jk.headings")
local tags = require("jk.tags")
local jk_home = vim.env.JK_HOME or (vim.fn.stdpath("config"))

--- Build path→title map from zk list (synchronous, fast)
local function title_map()
  local raw = vim.fn.system({ "zk", "list", "--quiet", "--format", "{{path}}\t{{title}}" })
  local map = {}
  for line in raw:gmatch("[^\n]+") do
    local path, title = line:match("^(.-)\t(.+)$")
    if path and title then map[path] = title end
  end
  return map
end

function M.titles()
  local tag_index = tags.build_index()
  local raw = vim.fn.system({ jk_home .. "/bin/search-titles" })
  local results = {}
  local path_for = {} -- ordinal → path

  for line in raw:gmatch("[^\n]+") do
    local title, path = line:match("^(.-)\t(.+)$")
    if title and path then
      if title == "" then title = vim.fn.fnamemodify(path, ":t:r") end
      -- Title entry
      results[#results + 1] = {
        title = title,
        path = path,
        is_section = false,
      }
      -- h2 section entries
      local h = headings.parse(path)
      for _, entry in ipairs(h) do
        if entry.level == 2 then
          results[#results + 1] = {
            title = title,
            section = entry.text,
            path = path,
            lnum = entry.lnum,
            is_section = true,
          }
        end
      end
    end
  end

  local base_sorter = conf.generic_sorter({})
  local original_scoring = base_sorter.scoring_function

  base_sorter.scoring_function = function(self, prompt, line, ...)
    local query, ptags = tags.parse_prompt(prompt)
    -- Hide sections when query is empty
    if query == "" and #ptags == 0 and line:find(" > ", 1, true) then
      return -1
    end
    -- Tag filtering
    if #ptags > 0 then
      local path = path_for[line]
      if path and not tags.file_matches(path, ptags, tag_index) then
        return -1
      end
    end
    return original_scoring(self, query, line, ...)
  end

  pickers.new({}, {
    prompt_title = "Notes",
    finder = finders.new_table({
      results = results,
      entry_maker = function(item)
        local display = headings.label(item.title, item.section)
        path_for[display] = item.path
        return {
          value = item,
          display = display,
          ordinal = display,
          filename = item.path,
          lnum = item.lnum,
        }
      end,
    }),
    sorter = base_sorter,
    previewer = conf.grep_previewer({}),
  }):find()
end

function M.grep()
  -- Precompute outside async context (vim.fn not safe in new_job entry_maker)
  local titles = title_map()
  local tag_index = tags.build_index()
  headings.clear_cache()

  local current_tags = {}

  -- Fallback: strip directory + extension from path (pure Lua, no vim.fn)
  local function basename_stem(path)
    local name = path:match("[^/]+$") or path
    return name:match("(.+)%.[^.]+$") or name
  end

  local function entry_maker(line)
    local file, lnum, col, text = line:match("^(.-):(%d+):(%d+):(.*)$")
    if not file then return nil end
    if not tags.file_matches(file, current_tags, tag_index) then return nil end
    lnum = tonumber(lnum)
    col = tonumber(col)

    local title = titles[file] or basename_stem(file)
    local heading = headings.at(file, lnum)
    local label = headings.label(title, heading)

    return {
      value = line,
      display = label .. ": " .. text,
      ordinal = label .. " " .. text,
      filename = file,
      lnum = lnum,
      col = col,
    }
  end

  pickers.new({}, {
    prompt_title = "Grep Notes",
    finder = finders.new_job(function(prompt)
      if not prompt or prompt == "" then return nil end
      local query, ptags = tags.parse_prompt(prompt)
      current_tags = ptags
      if query == "" then return nil end
      return { "rg", "--vimgrep", "--no-heading", "--smart-case", "--", query }
    end, entry_maker),
    previewer = conf.grep_previewer({}),
    sorter = sorters.highlighter_only({}),
  }):find()
end

function M.semantic()
  local script = jk_home .. "/bin/search-semantic"
  local tag_index = tags.build_index()

  local current_tags = {}

  local function entry_maker(line)
    local sim, file, lnum, heading, title = line:match("^(.-)\t(.-)\t(.-)\t(.-)\t(.*)$")
    if not sim then return nil end
    if not tags.file_matches(file, current_tags, tag_index) then return nil end
    lnum = tonumber(lnum) or 1
    local label = headings.label(
      title ~= "" and title or heading,
      title ~= "" and heading or nil
    )
    return {
      value = file,
      display = label,
      ordinal = label,
      filename = file,
      lnum = lnum,
    }
  end

  pickers.new({}, {
    prompt_title = "Semantic Search",
    finder = finders.new_job(function(prompt)
      if not prompt or prompt == "" then return nil end
      local query, ptags = tags.parse_prompt(prompt)
      current_tags = ptags
      if query == "" then return nil end
      return { script, query }
    end, entry_maker),
    previewer = conf.grep_previewer({}),
    sorter = sorters.highlighter_only({}),
  }):find()
end

return M
