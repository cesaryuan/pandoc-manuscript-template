-- Preserve DOCX-only metadata that Pandoc's writer cannot keep on its own.
--
-- This filter emits hidden WordprocessingML markers for:
-- 1. table attributes consumed by Python DOCX postprocessing,
-- 2. revised native Word display equations marked with `revision=true`,
-- 3. optional MathType equation-source markers used by MathType conversion.

local table_marker_prefix = "PMT_TABLE_METADATA:"
local equation_marker_prefix = "PMT_EQUATION_METADATA:"
local captioned_table_index = 0
local mathtype_markers_enabled = (os.getenv("PMT_ENABLE_MATHTYPE_MARKERS") or ""):lower() == "true"
local mathtype_counter = 0

local supported_table_keys = {
  ["cell_margin"] = true,
  ["cell-margin"] = true,
  ["cell_margin_top"] = true,
  ["cell-margin-top"] = true,
  ["cell_margin_bottom"] = true,
  ["cell-margin-bottom"] = true,
  ["cell_margin_left"] = true,
  ["cell-margin-left"] = true,
  ["cell_margin_right"] = true,
  ["cell-margin-right"] = true,
  ["cell_spacing"] = true,
  ["cell-spacing"] = true,
  ["row_height"] = true,
  ["row-height"] = true,
  ["revision_columns"] = true,
  ["revision-columns"] = true,
  ["revision_rows"] = true,
  ["revision-rows"] = true,
  ["alignment"] = true,
  ["autofit"] = true,
}

local function xml_escape(text)
  return text
    :gsub("&", "&amp;")
    :gsub("<", "&lt;")
    :gsub(">", "&gt;")
end

local function hidden_paragraph_marker(prefix, record)
  local payload = prefix .. pandoc.json.encode(record)
  local xml = '<w:p>'
    .. '<w:pPr><w:rPr><w:vanish/><w:specVanish/></w:rPr></w:pPr>'
    .. '<w:r><w:rPr><w:vanish/><w:specVanish/></w:rPr>'
    .. '<w:t>' .. xml_escape(payload) .. '</w:t></w:r>'
    .. '</w:p>'
  return pandoc.RawBlock("openxml", xml)
end

local function normalize_table_key(key)
  return key:gsub("-", "_")
end

local function table_caption_text(table)
  if table.caption == nil then
    return ""
  end
  return pandoc.utils.stringify(table.caption)
end

local function exported_table_attributes(table)
  local attrs = {}
  for key, value in pairs(table.attributes or {}) do
    if supported_table_keys[key] then
      attrs[normalize_table_key(key)] = tostring(value)
    end
  end
  return attrs
end

local function has_table_attributes(attrs)
  for _, _ in pairs(attrs) do
    return true
  end
  return false
end

local function inline_contains_display_math(inline)
  if inline.t == "Math" then
    return inline.mathtype == "DisplayMath"
  end
  if inline.t ~= "Span" then
    return false
  end
  for _, child in ipairs(inline.content) do
    if inline_contains_display_math(child) then
      return true
    end
  end
  return false
end

local function block_is_display_equation(block)
  if block.t ~= "Para" then
    return false
  end
  for _, inline in ipairs(block.content) do
    if inline_contains_display_math(inline) then
      return true
    end
  end
  return false
end

local function mathtype_marker_run(latex, kind)
  mathtype_counter = mathtype_counter + 1
  latex = latex:gsub("^%s+", ""):gsub("%s+$", "")
  local marker = "MTLATEX:" .. kind .. ":" .. latex
  local xml = table.concat({
    '<w:r>',
    '<w:rPr><w:vanish/></w:rPr>',
    '<w:t xml:space="preserve">',
    xml_escape(marker),
    '</w:t>',
    '</w:r>',
  })
  return pandoc.RawInline("openxml", xml)
end

function Table(table)
  if FORMAT ~= "docx" then
    return nil
  end

  local attrs = exported_table_attributes(table)
  if not has_table_attributes(attrs) then
    return nil
  end

  captioned_table_index = captioned_table_index + 1
  return {
    hidden_paragraph_marker(table_marker_prefix, {
      index = captioned_table_index,
      id = table.identifier or "",
      caption = table_caption_text(table),
      attributes = attrs,
    }),
    table,
  }
end

function Div(div)
  if FORMAT ~= "docx" then
    return nil
  end

  local revision_value = (div.attributes or {})["revision"]
  if revision_value == nil or revision_value:lower() ~= "true" then
    return nil
  end

  local blocks = {}
  for _, block in ipairs(div.content) do
    if block_is_display_equation(block) then
      table.insert(blocks, hidden_paragraph_marker(equation_marker_prefix, { revision = "true" }))
    end
    table.insert(blocks, block)
  end
  return blocks
end

function Math(math)
  if FORMAT ~= "docx" or not mathtype_markers_enabled then
    return nil
  end

  local kind = math.mathtype == "DisplayMath" and "display" or "inline"
  return { mathtype_marker_run(math.text, kind), math }
end
