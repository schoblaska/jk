-- Tag picker: browse tags, then show notes for selected tags with heading enrichment.
-- Usage:
--   require("jk.tag_picker")()              - open tag browser
--   require("jk.tag_picker")({ "tag1" })    - skip browser, show notes for tags

local function show_notes(tags)
  local h = require("jk.headings")
  local tp = require("telescope.pickers")
  local tf = require("telescope.finders")
  local sorters = require("telescope.sorters")
  local tconf = require("telescope.config").values

  local function entry_maker(line)
    local score, file, lnum, heading, title, linked_from =
      line:match("^(.-)\t(.-)\t(.-)\t(.-)\t(.-)\t(.*)$")
    if not score then return nil end
    lnum = tonumber(lnum) or 1
    heading = heading:gsub("^#+%s*", "")
    local label = h.label(
      title ~= "" and title or heading,
      title ~= "" and heading or nil
    )
    if linked_from and linked_from ~= "" then
      label = label .. " (via " .. linked_from .. ")"
    end
    return {
      value = file,
      display = label,
      ordinal = label,
      filename = file,
      lnum = lnum,
    }
  end

  local db = (vim.env.ZK_NOTEBOOK_DIR or vim.fn.getcwd()) .. "/.zk/search.db"
  local tag_where = table.concat(
    vim.tbl_map(function(t)
      return "(' ' || f.tags || ' ') LIKE '% " .. t .. " %'"
    end, tags),
    " AND "
  )
  local chunk_tag_where = "(" .. table.concat(
    vim.tbl_map(function(t)
      return "c.text LIKE '%#" .. t .. "%'"
    end, tags),
    " OR "
  ) .. ")"

  -- Empty prompt: show only tagged sections
  local browse_sql = string.format(
    "SELECT '1.000' || char(9) || c.file || char(9) || MIN(c.line) || char(9) || c.heading || char(9) || f.title || char(9) || '' FROM chunks c JOIN files f ON c.file = f.path WHERE %s AND %s GROUP BY c.file, c.heading ORDER BY f.title, MIN(c.line)",
    tag_where, chunk_tag_where
  )

  -- Search: FTS within tagged files (no semantic threshold to miss heading matches)
  local function search_sql(prompt)
    local terms = {}
    for word in prompt:gmatch("%S+") do
      for part in word:gmatch("%w+") do
        terms[#terms + 1] = part .. "*"
      end
    end
    if #terms == 0 then return nil end
    local fts = table.concat(terms, " ")
    return string.format(
      "SELECT printf('%%.3f', -bm25(chunks_fts, 5.0, 2.0, 1.0)) || char(9) || c.file || char(9) || c.line || char(9) || c.heading || char(9) || f.title || char(9) || '' FROM chunks_fts JOIN chunks c ON c.id = chunks_fts.rowid JOIN files f ON c.file = f.path WHERE chunks_fts MATCH '%s' AND %s ORDER BY bm25(chunks_fts, 5.0, 2.0, 1.0) LIMIT 50",
      fts, tag_where
    )
  end

  tp.new({}, {
    prompt_title = "Notes tagged #" .. table.concat(tags, ", #"),
    finder = tf.new_job(function(prompt)
      if not prompt or prompt == "" then
        return { "sqlite3", "-separator", "\t", db, browse_sql }
      end
      local fts_sql = search_sql(prompt)
      if not fts_sql then return nil end
      return { "sqlite3", "-separator", "\t", db, fts_sql }
    end, entry_maker),
    previewer = tconf.grep_previewer({}),
    sorter = sorters.highlighter_only({}),
  }):find()
end

local function browse_tags()
  local zk_api = require("zk.api")
  local notebook = vim.env.ZK_NOTEBOOK_DIR or vim.fn.getcwd()

  zk_api.tag.list(notebook, {}, function(err, tags)
    if err or not tags then return end
    vim.schedule(function()
      local tp = require("telescope.pickers")
      local tf = require("telescope.finders")
      local ta = require("telescope.actions")
      local tas = require("telescope.actions.state")
      local tau = require("telescope.actions.utils")
      local ted = require("telescope.pickers.entry_display")
      local tpv = require("telescope.previewers")
      local tconf = require("telescope.config").values

      local cache = {}

      local previewer = tpv.new_buffer_previewer({
        title = "Notes",
        define_preview = function(self, entry)
          local tag_name = entry.value.name
          local bufnr = self.state.bufnr
          if cache[tag_name] then
            if vim.api.nvim_buf_is_valid(bufnr) then
              vim.api.nvim_buf_set_lines(bufnr, 0, -1, false, cache[tag_name])
            end
            return
          end
          vim.api.nvim_buf_set_lines(bufnr, 0, -1, false, { "Loading…" })
          zk_api.list(notebook, { tags = { tag_name }, select = { "title", "rawContent" }, sort = { "title" } }, function(lerr, notes)
            if lerr or not notes then return end
            local h = require("jk.headings")
            local lines = {}
            for _, n in ipairs(notes) do
              local title = n.title or "(untitled)"
              if n.rawContent then
                local sections = h.sections_for(n.rawContent, "#" .. tag_name)
                if #sections == 0 then
                  lines[#lines + 1] = title
                else
                  for _, s in ipairs(sections) do
                    lines[#lines + 1] = h.label(title, s.text)
                  end
                end
              else
                lines[#lines + 1] = title
              end
            end
            cache[tag_name] = lines
            vim.schedule(function()
              if vim.api.nvim_buf_is_valid(bufnr) then
                vim.api.nvim_buf_set_lines(bufnr, 0, -1, false, lines)
              end
            end)
          end)
        end,
      })

      local displayer = ted.create({
        separator = " ",
        items = { { width = 4 }, { remaining = true } },
      })

      tp.new({}, {
        prompt_title = "Tags",
        finder = tf.new_table({
          results = tags,
          entry_maker = function(tag)
            return {
              value = tag,
              display = function(e)
                return displayer({
                  { e.value.note_count, "TelescopeResultsNumber" },
                  e.value.name,
                })
              end,
              ordinal = tag.name,
            }
          end,
        }),
        sorter = tconf.generic_sorter({}),
        previewer = previewer,
        attach_mappings = function(prompt_bufnr, _)
          ta.select_default:replace(function()
            local selection = {}
            tau.map_selections(prompt_bufnr, function(entry, _)
              table.insert(selection, entry.value.name)
            end)
            if #selection == 0 then
              local entry = tas.get_selected_entry()
              if entry then selection = { entry.value.name } end
            end
            ta.close(prompt_bufnr)
            if #selection == 0 then return end
            show_notes(selection)
          end)
          return true
        end,
      }):find()
    end)
  end)
end

return function(tags)
  if tags then
    show_notes(tags)
  else
    browse_tags()
  end
end
