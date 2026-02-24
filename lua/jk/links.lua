local function browse()
  local path = vim.api.nvim_buf_get_name(0)
  if path == "" then return end
  local zk_api = require("zk.api")
  local notebook = vim.env.ZK_NOTEBOOK_DIR
  local current_stem = vim.fn.expand("%:t:r")
  local results = {}
  local pending = 2

  local function done()
    pending = pending - 1
    if pending > 0 then return end
    vim.schedule(function()
      if #results == 0 then vim.notify("No links", vim.log.levels.INFO) return end
      table.sort(results, function(a, b) return a.display < b.display end)
      local pickers = require("telescope.pickers")
      local finders = require("telescope.finders")
      local conf = require("telescope.config").values
      local ta = require("telescope.actions")
      local tas = require("telescope.actions.state")
      pickers.new({}, {
        prompt_title = "Links",
        finder = finders.new_table({
          results = results,
          entry_maker = function(entry)
            return {
              value = entry,
              display = entry.display,
              ordinal = entry.ordinal or entry.display,
              filename = notebook .. "/" .. entry.path,
              lnum = entry.lnum or 1,
            }
          end,
        }),
        sorter = conf.generic_sorter({}),
        previewer = conf.grep_previewer({}),
        attach_mappings = function(prompt_bufnr, _)
          ta.select_default:replace(function()
            local entry = tas.get_selected_entry()
            if not entry then return end
            ta.close(prompt_bufnr)
            vim.cmd.edit(notebook .. "/" .. entry.value.path)
            if entry.value.backlink then
              vim.fn.search(vim.fn.escape(current_stem, "\\[].*^$~"), "cw")
            end
          end)
          return true
        end,
      }):find()
    end)
  end

  zk_api.list(notebook, { linkedBy = { path }, select = { "title", "path", "tags" } }, function(err, notes)
    if not err and notes then
      for _, n in ipairs(notes) do
        local title = n.title or vim.fn.fnamemodify(n.path, ":t:r")
        local tag_str = n.tags and #n.tags > 0 and " #" .. table.concat(n.tags, " #") or ""
        results[#results + 1] = {
          display = "→ " .. title,
          ordinal = "→ " .. title .. tag_str,
          path = n.path,
        }
      end
    end
    done()
  end)

  zk_api.list(notebook, { linkTo = { path }, select = { "title", "path", "tags" } }, function(err, notes)
    if not err and notes then
      local h = require("jk.headings")
      local escaped = vim.fn.escape(current_stem, "\\[].*^$~")
      for _, n in ipairs(notes) do
        local lnum = 1
        local abs = notebook .. "/" .. n.path
        local lines = vim.fn.readfile(abs)
        for i, line in ipairs(lines) do
          if line:find(escaped, 1, true) then lnum = i break end
        end
        local title = n.title or vim.fn.fnamemodify(n.path, ":t:r")
        local section = h.at(abs, lnum)
        local tag_str = n.tags and #n.tags > 0 and " #" .. table.concat(n.tags, " #") or ""
        results[#results + 1] = {
          display = "← " .. h.label(title, section),
          ordinal = "← " .. h.label(title, section) .. tag_str,
          path = n.path,
          backlink = true,
          lnum = lnum,
        }
      end
    end
    done()
  end)
end

return browse
