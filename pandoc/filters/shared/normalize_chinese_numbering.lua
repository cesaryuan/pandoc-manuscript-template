-- Normalize nested Chinese numbering before writer-specific rendering.
--
-- Headings and section references have the same semantic representation for
-- DOCX, HTML, and LaTeX, so this belongs in the Pandoc AST pipeline.

local enabled = false

-- Return whether an environment value represents an enabled flag.
local function is_truthy(value)
  if value == nil then
    return false
  end
  local text = tostring(value):lower()
  return text == "true" or text == "yes" or text == "1"
end

-- Return whether a metadata language value denotes Chinese text.
local function is_chinese_language(value)
  if value == nil then
    return false
  end
  local text = tostring(value):lower():gsub("_", "-")
  return text == "zh" or text == "zhcn" or text:match("^zh%-") ~= nil
end

-- Replace nested numeric hyphens with the dot notation used in Chinese text.
local function normalize_nested_number(text)
  return text:gsub("(%d+[%d%-]*%-%d+)", function(value)
    return value:gsub("%-", ".")
  end)
end

-- Normalize a section reference while preserving its surrounding text.
local function normalize_section_reference(text)
  return text:gsub("(节%s+)(%d+[%d%-]*%-%d+)", function(prefix, number)
    return prefix .. normalize_nested_number(number)
  end)
end

-- Normalize the leading nested number in a level 2+ heading.
local function normalize_header_inlines(inlines)
  for _, inline in ipairs(inlines) do
    if inline.t == "Str" then
      -- Preserve the DOCX behavior: only a leading nested number in level 2+
      -- headings is normalized, so ordinary dates in a title stay unchanged.
      local number, suffix = inline.text:match("^(%d+[%d%-]*%-%d+)(.*)$")
      if number ~= nil and (suffix == "" or suffix:match("^[%s%.。]")) then
        inline.text = normalize_nested_number(number) .. suffix
        return inlines
      end
    elseif inline.content ~= nil then
      normalize_header_inlines(inline.content)
    end
  end
  return inlines
end

-- Normalize section references in ordinary block inline content.
local function normalize_body_inlines(inlines)
  for index, inline in ipairs(inlines) do
    if inline.t == "Str" then
      inline.text = normalize_section_reference(inline.text)
    elseif inline.t == "Link" or inline.t == "Span" or inline.t == "Emph" or inline.t == "Strong" then
      inline.content = normalize_body_inlines(inline.content)
    end

    -- Markdown tokenizes `节 1-2` as Str, Space, Str.
    if inline.t == "Str" and inline.text:match("节$") ~= nil then
      local separator = inlines[index + 1]
      local number = inlines[index + 2]
      if separator ~= nil and separator.t == "Space" and number ~= nil and number.t == "Str" then
        number.text = normalize_nested_number(number.text)
      end
    end
  end
  return inlines
end

function Meta(meta)
  enabled = is_truthy(os.getenv("PMT_CHINESE_MODE"))
  if not enabled and meta.lang ~= nil then
    enabled = is_chinese_language(pandoc.utils.stringify(meta.lang))
  end
  return meta
end

function Pandoc(document)
  if not enabled then
    return document
  end

  return document:walk {
    Header = function(header)
      if header.level >= 2 then
        header.content = normalize_header_inlines(header.content)
      end
      return header
    end,
    Para = function(paragraph)
      paragraph.content = normalize_body_inlines(paragraph.content)
      return paragraph
    end,
    Plain = function(plain)
      plain.content = normalize_body_inlines(plain.content)
      return plain
    end,
    LineBlock = function(line_block)
      for index, line in ipairs(line_block.content) do
        line_block.content[index] = normalize_body_inlines(line)
      end
      return line_block
    end,
  }
end
