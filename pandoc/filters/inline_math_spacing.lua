-- Keep a paragraph containing one inline equation in Word's inline-math path.
--
-- Pandoc's AST already distinguishes InlineMath from DisplayMath. Appending a
-- Space here produces the preserved trailing text run that the old DOCX-only
-- postprocessor had to add after Word conversion.

-- Add a preserved trailing space only to paragraphs containing one inline formula.
function Para(paragraph)
  if #paragraph.content ~= 1 then
    return nil
  end

  local only_inline_math = paragraph.content[1].t == "Math"
    and paragraph.content[1].mathtype == "InlineMath"
  if not only_inline_math then
    return nil
  end

  paragraph.content[#paragraph.content + 1] = pandoc.Space()
  return paragraph
end
