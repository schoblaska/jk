return {
  "saghen/blink.cmp",
  version = "*",
  event = "InsertEnter",
  dependencies = {
    "ribru17/blink-cmp-spell",
  },
  opts = {
    sources = {
      default = { "lsp", "path", "buffer", "spell" },
      providers = {
        spell = {
          name = "Spell",
          module = "blink-cmp-spell",
          score_offset = -5,
        },
      },
    },
    keymap = {
      ["<Tab>"] = { "show", "select_and_accept", "fallback" },
      ["<S-Tab>"] = { "fallback" },
      ["<C-n>"] = { "select_next", "fallback" },
      ["<C-p>"] = { "select_prev", "fallback" },
      ["<C-y>"] = { "accept", "fallback" },
      ["<C-e>"] = { "cancel", "fallback" },
    },
    completion = {
      trigger = {
        show_on_insert = false,
        show_on_keyword = false,
        show_on_trigger_character = true,
      },
      menu = {
        draw = {
          columns = { { "kind_icon" }, { "label", "label_description", gap = 1 } },
        },
      },
      documentation = {
        auto_show = false,
      },
    },
  },
}
