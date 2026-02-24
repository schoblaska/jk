local M = {}

local cache = {} -- filepath -> [{lnum, text, raw, level}]

--- Parse all headings from a file, cached.
function M.parse(filepath)
  if cache[filepath] then return cache[filepath] end
  local entries = {}
  local f = io.open(filepath, "r")
  if f then
    local n = 0
    for line in f:lines() do
      n = n + 1
      local hashes, text = line:match("^(#+)%s+(.*)")
      if hashes then
        entries[#entries + 1] = {
          lnum = n,
          text = text,
          raw = line,
          level = #hashes,
        }
      end
    end
    f:close()
  end
  cache[filepath] = entries
  return entries
end

--- Nearest heading text at or before lnum.
function M.at(filepath, lnum)
  local headings = M.parse(filepath)
  local best = nil
  for _, h in ipairs(headings) do
    if h.lnum <= lnum then best = h.text else break end
  end
  return best
end

--- Find heading enclosing a target string in body.
--- Returns (text, raw) or nil for the first match.
function M.section_for(body, target)
  local heading, heading_raw = nil, nil
  for line in (body .. "\n"):gmatch("(.-)\n") do
    local hashes, text = line:match("^(#+)%s+(.*)")
    if hashes then
      heading = text
      heading_raw = line
    end
    if line:find(target, 1, true) then
      return heading, heading_raw
    end
  end
end

--- Find ALL headings enclosing occurrences of target in body.
--- Returns list of {text, raw, lnum} (deduplicated, preserving order).
function M.sections_for(body, target)
  local results = {}
  local seen = {}
  local stack = {} -- stack[level] = {text, raw, lnum}
  local depth = 0
  local n = 0
  for line in (body .. "\n"):gmatch("(.-)\n") do
    n = n + 1
    local hashes, text = line:match("^(#+)%s+(.*)")
    if hashes then
      local level = #hashes
      stack[level] = { text = text, raw = line, lnum = n }
      -- clear deeper levels
      for i = level + 1, 6 do stack[i] = nil end
      depth = level
    end
    if depth > 0 and line:find(target, 1, true) then
      -- build breadcrumb from stack
      local parts = {}
      for i = 2, 6 do
        if stack[i] then parts[#parts + 1] = stack[i].text end
      end
      local breadcrumb = #parts > 0 and table.concat(parts, " > ") or nil
      local key = breadcrumb or ""
      if not seen[key] then
        seen[key] = true
        results[#results + 1] = {
          text = breadcrumb,
          raw = stack[depth].raw,
          lnum = stack[depth].lnum,
        }
      end
    end
  end
  return results
end

--- Format "title > section" or just "title".
function M.label(title, section)
  if section and section ~= title then
    return title .. " > " .. section
  end
  return title
end

function M.clear_cache()
  cache = {}
end

return M
