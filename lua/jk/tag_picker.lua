-- Tag picker: browse tags, then show notes for selected tags with heading enrichment.
-- Usage:
--   require("jk.tag_picker")()              - open tag browser
--   require("jk.tag_picker")({ "tag1" })    - skip browser, show notes for tags

local function show_notes(tags)
  local zk_api = require("zk.api")
  local notebook = vim.env.ZK_NOTEBOOK_DIR or vim.fn.getcwd()
  local h = require("jk.headings")
  local tp = require("telescope.pickers")
  local tf = require("telescope.finders")
  local tconf = require("telescope.config").values

  zk_api.list(notebook, {
    tags = tags,
    select = { "title", "path", "rawContent" },
    sort = { "title" },
  }, function(err, notes)
    if err or not notes then return end
    local items = {}
    for _, n in ipairs(notes) do
      local title = n.title or vim.fn.fnamemodify(n.path, ":t:r")
      local added = false
      if n.rawContent then
        for _, tag_name in ipairs(tags) do
          local sections = h.sections_for(n.rawContent, "#" .. tag_name)
          for _, s in ipairs(sections) do
            items[#items + 1] = {
              display = h.label(title, s.text),
              path = n.path,
              lnum = s.lnum,
            }
            added = true
          end
        end
      end
      if not added then
        items[#items + 1] = { display = title, path = n.path, lnum = 1 }
      end
    end
    vim.schedule(function()
      tp.new({}, {
        prompt_title = "Notes tagged #" .. table.concat(tags, ", #"),
        finder = tf.new_table({
          results = items,
          entry_maker = function(item)
            return {
              value = item,
              display = item.display,
              ordinal = item.display,
              filename = notebook .. "/" .. item.path,
              lnum = item.lnum,
            }
          end,
        }),
        sorter = tconf.generic_sorter({}),
        previewer = tconf.grep_previewer({}),
      }):find()
    end)
  end)
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
