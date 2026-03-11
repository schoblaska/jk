return {
  "nvim-telescope/telescope.nvim",
  dependencies = {
    "nvim-lua/plenary.nvim",
    { "nvim-telescope/telescope-fzf-native.nvim", build = "make" },
  },
  config = function()
    local telescope = require("telescope")

    local actions = require("telescope.actions")
    local action_state = require("telescope.actions.state")

    local function insert_note_link(prompt_bufnr)
      local entry = action_state.get_selected_entry()
      actions.close(prompt_bufnr)
      if not entry or not entry.filename then return end

      local title
      if type(entry.value) == "table" and entry.value.title then
        title = entry.value.title
      else
        local f = io.open(entry.filename, "r")
        if f then
          local line = f:read("*l")
          f:close()
          title = line and line:match("^#%s+(.+)$")
        end
        title = title or vim.fn.fnamemodify(entry.filename, ":t:r")
      end

      -- Normalize both paths relative to notebook root
      local notebook = vim.env.ZK_NOTEBOOK_DIR or vim.fn.getcwd()
      local cur_abs = vim.api.nvim_buf_get_name(0)
      local cur_rel = cur_abs:sub(1, #notebook + 1) == notebook .. "/"
        and cur_abs:sub(#notebook + 2) or cur_abs
      local tgt_rel = entry.filename
      if tgt_rel:sub(1, #notebook + 1) == notebook .. "/" then
        tgt_rel = tgt_rel:sub(#notebook + 2)
      end

      local cur_dir = vim.fn.fnamemodify(cur_rel, ":h")
      local tgt_dir = vim.fn.fnamemodify(tgt_rel, ":h")
      local tgt_stem = vim.fn.fnamemodify(tgt_rel, ":t:r")
      if cur_dir == "." then cur_dir = "" end
      if tgt_dir == "." then tgt_dir = "" end

      local link_path
      if cur_dir == tgt_dir then
        link_path = tgt_stem
      else
        local from = cur_dir ~= "" and vim.split(cur_dir, "/") or {}
        local to = tgt_dir ~= "" and vim.split(tgt_dir, "/") or {}
        local common = 0
        for i = 1, math.min(#from, #to) do
          if from[i] == to[i] then common = i else break end
        end
        local parts = {}
        for _ = 1, #from - common do parts[#parts + 1] = ".." end
        for i = common + 1, #to do parts[#parts + 1] = to[i] end
        parts[#parts + 1] = tgt_stem
        link_path = table.concat(parts, "/")
      end

      local link = "[" .. title .. "](" .. link_path .. ")"
      local row, col = unpack(vim.api.nvim_win_get_cursor(0))
      vim.api.nvim_buf_set_text(0, row - 1, col, row - 1, col, { link })
      vim.api.nvim_win_set_cursor(0, { row, col + #link })
    end

    telescope.setup({
      defaults = {
        mappings = {
          i = {
            ["<C-q>"] = actions.send_selected_to_qflist + actions.open_qflist,
            ["<C-s>"] = actions.select_horizontal,
            ["<C-y>"] = insert_note_link,
          },
          n = {
            ["<C-q>"] = actions.send_selected_to_qflist + actions.open_qflist,
            ["<C-s>"] = actions.select_horizontal,
            ["<C-y>"] = insert_note_link,
          },
        },
        previewer = true,
        preview = { treesitter = true },
        prompt_prefix = " ",
        selection_caret = " ",
        sorting_strategy = "ascending",
        layout_strategy = "flex",
        preview_title = false,
        dynamic_preview_title = true,
        borderchars = { " ", " ", " ", " ", " ", " ", " ", " " },
        layout_config = {
          horizontal = {
            prompt_position = "top",
            preview_width = 0.5,
            preview_cutoff = 90,
          },
          vertical = {
            prompt_position = "top",
            preview_height = 0.60,
            preview_cutoff = 10,
          },
          flex = { flip_columns = 120 },
          anchor = "CENTER",
          width = { 0.92, max = 140 },
          height = { 0.90, max = 50 },
        },
      },
    })

    telescope.load_extension("fzf")

    vim.api.nvim_create_autocmd("User", {
      pattern = "TelescopePreviewerLoaded",
      callback = function()
        vim.wo.wrap = true
        vim.wo.linebreak = true
        vim.fn.matchadd("ZkTag", "\\#[[:alnum:]][[:alnum:]_-]*")
      end,
    })
  end,
}
