-- Insert hidden DOCX markers that carry the original LaTeX for each math node.
-- The MathType postprocessor reads these markers to bind each OMML node to its
-- own LaTeX source instead of relying on global formula order.

local counter = 0

local function xml_escape(value)
  return value
    :gsub("&", "&amp;")
    :gsub("<", "&lt;")
    :gsub(">", "&gt;")
end

local function marker_run(latex, kind)
  counter = counter + 1
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

function Math(math)
  if FORMAT ~= "docx" then
    return nil
  end

  local kind = math.mathtype == "DisplayMath" and "display" or "inline"
  return { marker_run(math.text, kind), math }
end
