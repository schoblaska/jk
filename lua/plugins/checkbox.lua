return {
  "opdavies/toggle-checkbox.nvim",
  keys = {
    {
      "<leader>x",
      function() require("toggle-checkbox").toggle() end,
      desc = "Toggle task checkbox",
    },
  },
}
