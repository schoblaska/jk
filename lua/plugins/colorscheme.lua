return {
  {
    "EdenEast/nightfox.nvim",
    lazy = false,
    priority = 1000,
    config = function()
      require("nightfox").setup({
        options = {
          styles = {
            comments = "italic",
          },
        },
        palettes = {
          duskfox = {
            bg0 = "#1B1928",
          },
        },
        groups = {
          all = {
            TabLineSel = { fg = "fg1", bg = "bg1" },
            TabLineSeparatorSel = { fg = "palette.yellow" },
            TabLineSeparator = { fg = "palette.comment", bg = "bg0" },
            TabLine = { fg = "palette.comment", bg = "bg0" },
            TabLineFill = { bg = "bg0" },
            ["@markup.link.label"] = { fg = "palette.pink" },
            ["@markup.link.url"] = { fg = "palette.comment" },
            ZkTag = { fg = "palette.pink", style = "italic" },
            NormalFloat = { bg = "bg0" },
            FloatBorder = { bg = "bg0", fg = "bg0" },
            SnacksNotifierHistory = { bg = "bg0" },
            SnacksNotifier = { bg = "bg0" },
            SnacksNotifierBorderInfo = { bg = "bg0", fg = "bg0" },
            SnacksNotifierBorderWarn = { bg = "bg0", fg = "bg0" },
            SnacksNotifierBorderError = { bg = "bg0", fg = "bg0" },
            SnacksNotifierInfo = { bg = "bg0", fg = "fg1" },
            SnacksNotifierWarn = { bg = "bg0", fg = "fg1" },
            SnacksNotifierError = { bg = "bg0", fg = "fg1" },
            SnacksNotifierIconInfo = { bg = "bg0", fg = "palette.cyan" },
            SnacksNotifierIconWarn = { bg = "bg0", fg = "palette.yellow" },
            SnacksNotifierIconError = { bg = "bg0", fg = "palette.red" },
            SnacksNotifierTitleInfo = { fg = "palette.cyan" },
            SnacksNotifierTitleWarn = { fg = "palette.yellow" },
            SnacksNotifierTitleError = { fg = "palette.red" },
            Pmenu = { bg = "bg0" },
            PmenuSel = { bg = "bg2" },
            NormalDark = { bg = "bg0" },
            CursorLineDark = { bg = "bg1" },
            TelescopeNormal = { bg = "bg1" },
            TelescopeBorder = { bg = "bg1", fg = "bg1" },
            TelescopePromptNormal = { bg = "bg1" },
            TelescopePromptBorder = { bg = "bg1", fg = "bg1" },
            TelescopePromptPrefix = { bg = "bg1" },
            TelescopePromptTitle = { bg = "bg1", fg = "palette.comment" },
            TelescopeResultsNormal = { bg = "bg1" },
            TelescopeResultsBorder = { bg = "bg1", fg = "bg1" },
            TelescopePreviewNormal = { bg = "bg0" },
            TelescopePreviewBorder = { bg = "bg0", fg = "bg0" },
            TelescopePreviewTitle = { bg = "bg0", fg = "palette.comment" },
            TelescopeMatching = { link = "DiffChange" },
            TelescopeSelection = { link = "DiffChange" },
            TelescopePreviewLine = { link = "DiffChange" },
            LspReferenceText = { link = "DiffChange" },
            LspReferenceRead = { link = "DiffChange" },
            LspReferenceWrite = { link = "DiffChange" },
            AerialInterfaceIcon = { link = "@markup.heading" },
            AerialInterface = { link = "@markup.heading" },
            AerialLine = { link = "DiffChange" },
            IncSearch = { link = "@comment.warning" },
            Search = { link = "DiffChange" },
            HlSearchNear = { link = "@comment.warning" },
            HlSearchLensNear = { link = "@comment.warning" },
          },
          duskfox = {
            PmenuSel = { bg = "#353051" },
          },
          dayfox = {
            CursorLine = { bg = "bg0" },
          },
        },
      })
    end,
  },

  {
    "f-person/auto-dark-mode.nvim",
    lazy = false,
    priority = 999,
    opts = {
      update_interval = 1000,
      set_dark_mode = function()
        vim.cmd("colorscheme duskfox")
      end,
      set_light_mode = function()
        vim.cmd("colorscheme dayfox")
      end,
    },
  },
}
