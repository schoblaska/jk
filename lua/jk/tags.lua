local M = {}

--- Extract #tag tokens from prompt, return stripped query + tag list
function M.parse_prompt(prompt)
  local tags = {}
  local query = prompt:gsub("#([%w_-]+)", function(tag)
    tags[#tags + 1] = tag
    return ""
  end)
  query = query:match("^%s*(.-)%s*$") or ""
  return query, tags
end

--- Scan frontmatter of all .md files, return {path = {tag = true}}
function M.build_index()
  local files = vim.fn.glob("**/*.md", false, true)
  local index = {}
  for _, path in ipairs(files) do
    local f = io.open(path, "r")
    if f then
      local tag_set = {}
      for _ = 1, 10 do
        local line = f:read("*l")
        if not line then break end
        local tags_str = line:match("^tags:%s*(.+)$")
        if tags_str then
          for tag in tags_str:gmatch("#([%w_-]+)") do
            tag_set[tag] = true
          end
          break
        end
      end
      f:close()
      index[path] = tag_set
    end
  end
  return index
end

--- Returns true if path has ALL required tags (AND logic)
function M.file_matches(path, tags, index)
  if #tags == 0 then return true end
  local file_tags = index[path]
  if not file_tags then return false end
  for _, tag in ipairs(tags) do
    if not file_tags[tag] then return false end
  end
  return true
end

return M
