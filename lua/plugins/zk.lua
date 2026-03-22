local function journal_sibling(direction)
  local name = vim.fn.expand("%:t")
  if not name:match("^%d%d%d%d%-%d%d%-%d%d%.md$") then return end
  local notebook = vim.env.ZK_NOTEBOOK_DIR or vim.fn.getcwd()
  local entries = {}
  for file in vim.fs.dir(notebook) do
    if file:match("^%d%d%d%d%-%d%d%-%d%d%.md$") then
      entries[#entries + 1] = file
    end
  end
  table.sort(entries)
  for i, f in ipairs(entries) do
    if f == name then
      local target = entries[i + direction]
      if target then vim.cmd.edit(notebook .. "/" .. target) end
      return
    end
  end
end

return {
  "zk-org/zk-nvim",
  lazy = false,
  config = function()
    require("zk").setup({
      picker = "telescope",
      lsp = {
        config = {
          cmd = { "zk", "lsp" },
          name = "zk",
        },
        auto_attach = {
          enabled = true,
          filetypes = { "markdown" },
        },
      },
    })

    local ns = vim.api.nvim_create_namespace("zk_tags")
    vim.api.nvim_set_decoration_provider(ns, {
      on_win = function(_, _, bufnr)
        if vim.bo[bufnr].filetype ~= "markdown" then return false end
      end,
      on_line = function(_, _, bufnr, row)
        local line = vim.api.nvim_buf_get_lines(bufnr, row, row + 1, false)[1]
        if not line then return end
        local col = 1
        while col <= #line do
          local start, finish = line:find("#[%w][%w_%-]*", col)
          if not start then break end
          vim.api.nvim_buf_set_extmark(bufnr, ns, row, start - 1, {
            end_col = finish,
            hl_group = "ZkTag",
            ephemeral = true,
          })
          col = finish + 1
        end
      end,
    })

    -- Force zk LSP to refresh diagnostics (dead-link checks) on buffer focus
    vim.api.nvim_create_autocmd("BufEnter", {
      pattern = "*.md",
      callback = function(ev)
        local uri = vim.uri_from_bufnr(ev.buf)
        for _, client in ipairs(vim.lsp.get_clients({ name = "zk", bufnr = ev.buf })) do
          client:notify("textDocument/didClose", {
            textDocument = { uri = uri },
          })
          client:notify("textDocument/didOpen", {
            textDocument = {
              uri = uri,
              languageId = "markdown",
              version = 0,
              text = table.concat(vim.api.nvim_buf_get_lines(ev.buf, 0, -1, false), "\n"),
            },
          })
        end
      end,
    })
  end,
  keys = {
    { "<leader>n", function()
      vim.ui.input({ prompt = "Title: " }, function(input)
        if input and input ~= "" then
          require("zk.commands").get("ZkNew")({ title = require("jk.follow").title_case(input) })
        end
      end)
    end, desc = "New note" },

    { "<leader>n", function()
      vim.cmd("noautocmd normal! \27")
      local util = require("zk.util")
      local location = util.get_lsp_location_from_selection()
      local text = util.get_selected_text()
      if text and text ~= "" then
        text = text:gsub("%s+$", ""):gsub("^%s+", "")
        require("zk.commands").get("ZkNew")({
          title = require("jk.follow").title_case(text),
          insertLinkAtLocation = location,
        })
      end
    end, mode = "v", desc = "New note from selection" },

    { "<leader>l", function()
      vim.cmd("noautocmd normal! \27")
      local start_pos = vim.api.nvim_buf_get_mark(0, "<")
      local end_pos = vim.api.nvim_buf_get_mark(0, ">")
      local line = vim.api.nvim_get_current_line()
      local text = line:sub(start_pos[2] + 1, end_pos[2] + 1)
      local replacement = "[" .. text .. "]("
      vim.api.nvim_buf_set_text(0, start_pos[1] - 1, start_pos[2], end_pos[1] - 1, end_pos[2] + 1, { replacement })
      local paren_col = start_pos[2] + #replacement - 1
      vim.api.nvim_win_set_cursor(0, { start_pos[1], paren_col })
      vim.fn.feedkeys("a(", "n")
    end, mode = "v", ft = "markdown", desc = "Link selection" },

    { "<leader>j", "<Cmd>ZkNew { group = 'journal' }<CR>", desc = "Journal" },
    { "<leader>o", function() require("jk.search").titles() end, desc = "Open note" },
    { "<leader>t", function() require("jk.tag_picker")() end, desc = "Tags" },
    { "<leader>f", function() require("jk.search").search() end, desc = "Search notes" },
    { "<leader>f", ":'<,'>ZkMatch<CR>", mode = "v", desc = "Search notes (selection)" },
    { "<leader>r", "<Cmd>ZkNotes { sort = { 'modified' } }<CR>", desc = "Recent notes" },
    { "<leader>e", "<Cmd>AerialToggle float<CR>", desc = "Outline", ft = "markdown" },

    { "<CR>", function() require("jk.follow").follow_link("edit") end, ft = "markdown", desc = "Follow link / tag" },
    { "<C-v>", function() require("jk.follow").follow_link("vsplit") end, ft = "markdown", desc = "Follow link (vsplit)" },
    { "<C-s>", function() require("jk.follow").follow_link("split") end, ft = "markdown", desc = "Follow link (split)" },
    { "<C-t>", function() require("jk.follow").follow_link("tabedit") end, ft = "markdown", desc = "Follow link (tab)" },

    { "K", function() require("jk.preview").hover() end, ft = "markdown", desc = "Hover / preview backlink" },

    { "]l", function() vim.fn.search("\\[\\[\\|\\](", "w") end, ft = "markdown", desc = "Next link" },
    { "[l", function() vim.fn.search("\\[\\[\\|\\](", "bw") end, ft = "markdown", desc = "Prev link" },
    { "]]", function() journal_sibling(1) end, ft = "markdown", desc = "Next journal" },
    { "[[", function() journal_sibling(-1) end, ft = "markdown", desc = "Prev journal" },

    { "<leader>l", function() require("jk.links")() end, desc = "Links" },

    { "<leader>d", function()
      local bufnr = vim.api.nvim_get_current_buf()
      local path = vim.api.nvim_buf_get_name(bufnr)
      if path == "" or not path:match("%.md$") then return end
      local notebook = vim.env.ZK_NOTEBOOK_DIR or vim.fn.getcwd()
      local rel = path:sub(#notebook + 2)
      if vim.fn.confirm("Delete " .. rel .. "?", "&Yes\n&No", 2) ~= 1 then return end
      local db = notebook .. "/.zk/search.db"
      if vim.fn.filereadable(db) == 1 then
        vim.fn.system({ "sqlite3", db, "DELETE FROM chunks WHERE file = '" .. rel:gsub("'", "''") .. "';" })
      end
      vim.fn.delete(path)
      local alt = vim.fn.bufnr("#")
      if alt ~= -1 and alt ~= bufnr and vim.api.nvim_buf_is_valid(alt) then
        vim.cmd.buffer(alt)
      else
        local fallback
        for _, b in ipairs(vim.api.nvim_list_bufs()) do
          if b ~= bufnr and vim.api.nvim_buf_is_loaded(b) then
            local name = vim.api.nvim_buf_get_name(b)
            if name:match("%.md$") then
              fallback = b
              break
            end
          end
        end
        vim.cmd.buffer(fallback or vim.api.nvim_create_buf(true, false))
      end
      vim.cmd("bdelete! " .. bufnr)
      vim.fn.jobstart({ "zk", "index", "--quiet" }, {
        cwd = notebook,
        on_exit = function()
          vim.fn.jobstart({ "zk", "gen-index" }, { cwd = notebook })
        end,
      })
      vim.notify("Deleted " .. rel, vim.log.levels.INFO)
    end, desc = "Delete note" },

    { "<leader>X", function() require("jk.todos")() end, desc = "TODOs" },
    { "<leader>a", function() require("jk.claude").toggle() end, desc = "Toggle Claude" },
    { "<leader>A", function() require("jk.claude").open() end, desc = "New Claude" },
  },
}
