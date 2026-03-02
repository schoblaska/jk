vim.g.mapleader = " "
vim.g.maplocalleader = " "

local opt = vim.opt

-- Files
opt.backup = false
opt.writebackup = false
opt.swapfile = false
opt.undofile = false

-- Clipboard
opt.clipboard = ""

-- UI
opt.cursorline = true
opt.termguicolors = true
opt.number = true
opt.relativenumber = false
opt.numberwidth = 1
opt.signcolumn = "no"
opt.showmode = false
opt.showcmd = false
opt.showtabline = 1
opt.pumheight = 10
opt.colorcolumn = "99999"
opt.conceallevel = 0
opt.winbar = "%{%v:lua.jk_winbar()%}"
opt.laststatus = 3
opt.cmdheight = 0

-- Editing
opt.expandtab = true
opt.shiftwidth = 2
opt.tabstop = 2
opt.smartindent = true
opt.wrap = true
opt.linebreak = true
opt.breakindent = true
opt.smoothscroll = true

-- Search
opt.ignorecase = true
opt.smartcase = true
opt.hlsearch = true

-- Scrolling
opt.scrolloff = 12
opt.sidescrolloff = 8

-- Splits
opt.splitbelow = true
opt.splitright = true

-- Mouse
opt.mouse = "a"
opt.mousescroll = "ver:6,hor:6"

-- Completion
opt.completeopt = { "menuone", "noselect" }

-- Timing
opt.timeoutlen = 500
opt.ttimeoutlen = 5
opt.updatetime = 300

-- Misc
opt.hidden = true
opt.fileencoding = "utf-8"
opt.title = true
opt.titlestring = "%<%F - jk"

-- Neovide
if vim.g.neovide then
  vim.g.neovide_cursor_animation_length = 0
  vim.g.neovide_cursor_animate_command_line = false
  vim.g.neovide_cursor_smooth_blink = false
  vim.g.neovide_cursor_vfx_mode = ""
  vim.g.neovide_scroll_animation_length = 0.1
  vim.g.neovide_position_animation_length = 0
  vim.g.neovide_scale_factor = 1.1
  vim.g.neovide_padding_top = 4
  vim.g.neovide_padding_bottom = 4
  vim.g.neovide_padding_left = 8
  vim.g.neovide_padding_right = 8
end

-- Spelling (z= suggestions without squiggles)
opt.spell = true
opt.spelllang = { "en_us" }
vim.api.nvim_create_autocmd("ColorScheme", {
  callback = function()
    vim.api.nvim_set_hl(0, "SpellBad", {})
    vim.api.nvim_set_hl(0, "SpellCap", {})
    vim.api.nvim_set_hl(0, "SpellRare", {})
    vim.api.nvim_set_hl(0, "SpellLocal", {})

    -- Strip italic from every highlight group
    for _, name in ipairs(vim.fn.getcompletion("", "highlight")) do
      local hl = vim.api.nvim_get_hl(0, { name = name })
      if hl.italic then
        hl.italic = false
        vim.api.nvim_set_hl(0, name, hl)
      end
    end
  end,
})

-- Markdown
vim.g.markdown_recommended_style = 0

-- Floats
opt.winborder = "solid"

-- Project-local ShaDa
local shada_dir = vim.fn.stdpath("data") .. "/shada-projects"
local project_hash = vim.fn.sha256(vim.fn.getcwd()):sub(1, 12)
vim.fn.mkdir(shada_dir, "p")
vim.o.shadafile = shada_dir .. "/" .. project_hash .. ".shada"
