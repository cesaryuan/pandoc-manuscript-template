-- Convert Markdown italics to Word's "Emphasis" character style for DOCX output.
-- This keeps the manuscript syntax simple while letting the reference DOCX style
-- control how emphasized text appears in Word.

local EMPHASIS_STYLE = "Emphasis Char"

function Emph(elem)
  -- Pandoc writes custom-style spans as DOCX character styles; replacing Emph
  -- avoids stacking direct italic formatting on top of the named Word style.
  return pandoc.Span(elem.content, pandoc.Attr("", {}, {["custom-style"] = EMPHASIS_STYLE}))
end
