-- Extract `revision=true` from display-equation trailing attributes before pandoc-crossref.
--
-- Pandoc parses `$$...$$ {#eq:label revision=true}` as a display-math paragraph
-- followed by plain inline text, so the equation never receives a real AST
-- attribute. This filter strips the custom `revision=...` token from that
-- trailing attribute text, preserves the remaining label/class tokens for
-- pandoc-crossref, and wraps revised equations in a Div that later DOCX-only
-- filters can detect.

local function trim(text)
  return (text:gsub("^%s+", ""):gsub("%s+$", ""))
end

local function is_display_math_inline(inline)
  return inline.t == "Math" and inline.mathtype == "DisplayMath"
end

local function normalize_attr_value(value)
  local normalized = trim(value)
  normalized = normalized:gsub('^"(.*)"$', "%1")
  normalized = normalized:gsub("^'(.*)'$", "%1")
  return normalized:lower()
end

local function extract_revision_attr(text)
  local trimmed = trim(text)
  if not trimmed:match("^%b{}$") then
    return nil
  end

  local inner = trim(trimmed:sub(2, -2))
  if inner == "" then
    return nil
  end

  local preserved_tokens = {}
  local revision_value = nil

  for token in inner:gmatch("%S+") do
    local key, value = token:match("^([%w_-]+)=(.+)$")
    if key == "revision" then
      revision_value = normalize_attr_value(value)
    else
      table.insert(preserved_tokens, token)
    end
  end

  if revision_value == nil then
    return nil
  end

  local preserved = table.concat(preserved_tokens, " ")
  if preserved ~= "" then
    preserved = "{" .. preserved .. "}"
  else
    preserved = nil
  end

  return {
    preserved = preserved,
    enabled = revision_value == "true",
  }
end

function Para(para)
  if #para.content == 0 or not is_display_math_inline(para.content[1]) then
    return nil
  end

  local trailing = {}
  for i = 2, #para.content do
    trailing[#trailing + 1] = para.content[i]
  end
  local revision = extract_revision_attr(pandoc.utils.stringify(trailing))
  if revision == nil then
    return nil
  end

  local rewritten = { para.content[1] }
  if revision.preserved ~= nil then
    rewritten[#rewritten + 1] = pandoc.Space()
    rewritten[#rewritten + 1] = pandoc.Str(revision.preserved)
  end

  local rewritten_para = pandoc.Para(rewritten)
  if not revision.enabled then
    return rewritten_para
  end

  return pandoc.Div(
    { rewritten_para },
    pandoc.Attr("", {}, { { "revision", "true" } })
  )
end
