-- Preserve Pandoc table attributes inside the intermediate DOCX.
--
-- Pandoc's DOCX writer drops arbitrary table attributes, so this filter writes
-- a hidden WordprocessingML marker before each attributed table. The Python
-- postprocessor reads and removes the marker before saving the final DOCX.

local marker_prefix = "PMT_TABLE_METADATA:"
local captioned_table_index = 0

local supported_keys = {
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
  ["alignment"] = true,
  ["autofit"] = true,
}

-- Normalize Pandoc attribute names to the Python postprocessor's key names.
local function normalize_key(key)
  return key:gsub("-", "_")
end

-- Escape marker JSON so it is valid text inside w:t.
local function xml_escape(text)
  return text
    :gsub("&", "&amp;")
    :gsub("<", "&lt;")
    :gsub(">", "&gt;")
end

-- Return the table caption as plain text for debugging and matching.
local function caption_text(table)
  if table.caption == nil then
    return ""
  end
  return pandoc.utils.stringify(table.caption)
end

-- Return whether a Table block corresponds to a captioned source table.
local function has_caption(table)
  return caption_text(table) ~= "" or (table.identifier or ""):match("^tbl:")
end

-- Return only attributes handled by the DOCX table postprocessor.
local function table_attributes(table)
  local attrs = {}
  for key, value in pairs(table.attributes or {}) do
    if supported_keys[key] then
      attrs[normalize_key(key)] = tostring(value)
    end
  end
  return attrs
end

-- Return whether a table has attributes that need DOCX postprocessing.
local function has_table_attributes(attrs)
  for _, _ in pairs(attrs) do
    return true
  end
  return false
end

-- Create a hidden paragraph carrying one table metadata record.
local function marker_block(record)
  local payload = marker_prefix .. pandoc.json.encode(record)
  local xml = '<w:p>'
    .. '<w:pPr><w:rPr><w:vanish/><w:specVanish/></w:rPr></w:pPr>'
    .. '<w:r><w:rPr><w:vanish/><w:specVanish/></w:rPr>'
    .. '<w:t>' .. xml_escape(payload) .. '</w:t></w:r>'
    .. '</w:p>'
  return pandoc.RawBlock("openxml", xml)
end

function Table(table)
  if not has_caption(table) then
    return nil
  end

  local attrs = table_attributes(table)
  if not has_table_attributes(attrs) then
    return nil
  end

  captioned_table_index = captioned_table_index + 1
  return {
    marker_block({
      index = captioned_table_index,
      id = table.identifier or "",
      caption = caption_text(table),
      attributes = attrs,
    }),
    table,
  }
end
