-- Custom tabline: show filename, or ai/filename for ai subdir
function _G.jk_tabline()
  local s = ""
  for i = 1, vim.fn.tabpagenr("$") do
    local winnr = vim.fn.tabpagewinnr(i)
    local bufnr = vim.fn.tabpagebuflist(i)[winnr]
    local name = vim.fn.bufname(bufnr)
    local label

    if name == "" then
      label = "[No Name]"
    else
      local tail = vim.fn.fnamemodify(name, ":t")
      local parent = vim.fn.fnamemodify(name, ":h:t")
      label = (parent == "ai") and ("ai/" .. tail) or tail

      if tail:match("%.md$") and vim.api.nvim_buf_is_loaded(bufnr) then
        local first_line = vim.api.nvim_buf_get_lines(bufnr, 0, 1, false)[1] or ""
        local title = first_line:match("^#%s+(.+)")
        if title then
          label = (parent == "ai") and ("ai/" .. title) or title
        end
      end
    end

    local is_sel = (i == vim.fn.tabpagenr())
    local sep_hl = is_sel and "%#TabLineSeparatorSel#" or "%#TabLineSeparator#"
    local tab_hl = is_sel and "%#TabLineSel#" or "%#TabLine#"
    s = s .. sep_hl .. " " .. tab_hl .. " " .. label .. " "
  end
  return s .. "%#TabLineFill#"
end

vim.o.tabline = "%!v:lua.jk_tabline()"

-- Winbar: show filename without extension for markdown, filepath otherwise
function _G.jk_winbar()
  if vim.bo.buftype ~= "" then return "" end
  local name = vim.fn.expand("%:t")
  if name:match("%.md$") then
    local stem = name:gsub("%.md$", "")
    local first_line = vim.api.nvim_buf_get_lines(0, 0, 1, false)[1] or ""
    local title = first_line:match("^#%s+(.+)")
    if title then
      return "%=[" .. stem .. "] " .. title
    end
    return "%=" .. stem
  end
  return "%=%f"
end

-- Only show cursorline in active window
vim.api.nvim_create_autocmd({ "WinEnter", "BufWinEnter" }, {
  pattern = "*",
  command = "setlocal cursorline"
})

vim.api.nvim_create_autocmd("WinLeave", {
  pattern = "*",
  command = "setlocal nocursorline"
})

-- Hide winbar on terminal buffers
vim.api.nvim_create_autocmd("TermOpen", {
  callback = function()
    vim.wo.winbar = ""
  end,
})

-- Startup layout: motd (or daily note)
vim.api.nvim_create_autocmd("VimEnter", {
  once = true,
  callback = function()
    if vim.fn.argc() > 0 then return end

    local notebook = vim.env.ZK_NOTEBOOK_DIR
    if not notebook then return end

    local function open_motd()
      -- Ensure we target a normal window, not a lazy.nvim float
      for _, win in ipairs(vim.api.nvim_list_wins()) do
        if vim.api.nvim_win_get_config(win).relative == "" then
          vim.api.nvim_set_current_win(win)
          break
        end
      end

      local motd = notebook .. "/motd.md"
      if vim.fn.filereadable(motd) == 1 then
        vim.cmd("edit " .. vim.fn.fnameescape(motd))
      else
        require("zk.commands").get("ZkNew")({ group = "journal" })
      end
      vim.cmd("stopinsert")
    end

    -- Defer slightly to let any lazy.nvim install UI open first,
    -- then we target the correct (non-floating) window
    vim.defer_fn(open_motd, 50)
  end,
})

-- Re-index on save
local notebook = vim.env.ZK_NOTEBOOK_DIR

if notebook then
  vim.api.nvim_create_autocmd("BufWritePost", {
    pattern = notebook .. "/*.md",
    callback = function(ev)
      if vim.fn.fnamemodify(ev.file, ":t") == "index.md" then return end
      if vim.env.JK_HOME then
        vim.fn.jobstart({ vim.env.JK_HOME .. "/bin/reindex", ev.file }, { cwd = notebook })
      end
    end,
  })
end
