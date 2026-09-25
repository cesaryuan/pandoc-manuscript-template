-- Apply the Word "Figure" paragraph style to standalone captionless images.
--
-- Pandoc parses `![](image.png){width=50%}` as a normal paragraph containing an
-- Image inline, not as a Figure block. Without this DOCX-only filter, Word uses
-- the body-text paragraph style for that image paragraph. The filter detects
-- paragraphs that contain exactly one image with an empty caption and wraps that
-- paragraph in a custom-style Div so Pandoc's DOCX writer emits the named Figure
-- paragraph style while preserving the image attributes.

local FIGURE_STYLE = "Figure"

local function is_ignorable_inline(inline)
  -- Permit harmless whitespace around a standalone image without matching prose.
  return inline.t == "Space" or inline.t == "SoftBreak" or inline.t == "LineBreak"
end

local function image_caption_text(image)
  -- Empty alt text is the specific Pandoc shape that otherwise remains Body Text.
  return pandoc.utils.stringify(image.caption or {})
end

local function standalone_captionless_image(para)
  -- Return the only image when the paragraph is an uncaptioned image block.
  local image = nil
  for _, inline in ipairs(para.content) do
    if inline.t == "Image" then
      if image ~= nil then
        return nil
      end
      image = inline
    elseif not is_ignorable_inline(inline) then
      return nil
    end
  end

  if image == nil or image_caption_text(image) ~= "" then
    return nil
  end
  return image
end

function Para(para)
  if FORMAT ~= "docx" or standalone_captionless_image(para) == nil then
    return nil
  end

  return pandoc.Div(
    { para },
    pandoc.Attr("", {}, { ["custom-style"] = FIGURE_STYLE })
  )
end
