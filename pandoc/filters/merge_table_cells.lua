-- Merge table cells marked with the Papper placeholders `!<!` and `!^!`.
--
-- Table cell spans are part of Pandoc's AST, so this transformation is shared
-- by DOCX, HTML, LaTeX, and other writers that support table spans.

-- Return trimmed visible text for one Pandoc table cell.
local function cell_text(cell)
  return pandoc.utils.stringify(cell.contents):match("^%s*(.-)%s*$")
end

-- Add a cell span while respecting Pandoc's default span value of one.
local function add_span(cell, field, amount)
  local current = cell[field] or 1
  cell[field] = current + amount
end

-- Consume left-merge markers in one row from right to left.
local function merge_left(row)
  local count = 0
  for index = #row, 2, -1 do
    local marker = row[index]
    if cell_text(marker) == "!<!" then
      local target = row[index - 1]
      add_span(target, "col_span", marker.col_span or 1)
      table.remove(row, index)
      count = count + 1
    end
  end
  return count
end

-- Consume upward-merge markers across the accumulated table rows.
local function merge_up(rows)
  local count = 0
  for row_index = #rows, 2, -1 do
    local row = rows[row_index]
    local above = rows[row_index - 1]
    for cell_index = #row.cells, 1, -1 do
      local marker = row.cells[cell_index]
      if cell_text(marker) == "!^!" and above.cells[cell_index] ~= nil then
        add_span(above.cells[cell_index], "row_span", marker.row_span or 1)
        table.remove(row.cells, cell_index)
        count = count + 1
      end
    end
  end
  return count
end

-- Read rows from either the row-based or body-based table section API.
local function section_rows(section)
  -- TableHead/TableFoot expose rows; TableBody exposes body.
  if section == nil then
    return {}
  end
  return section.rows or section.body or {}
end

function Table(tbl)
  local left = 0
  local up = 0
  local all_rows = {}

  for _, row in ipairs(section_rows(tbl.head)) do
    left = left + merge_left(row.cells)
    all_rows[#all_rows + 1] = row
  end

  for _, body in ipairs(tbl.bodies or {}) do
    for _, row in ipairs(section_rows(body)) do
      left = left + merge_left(row.cells)
      all_rows[#all_rows + 1] = row
    end
  end

  for _, row in ipairs(section_rows(tbl.foot)) do
    left = left + merge_left(row.cells)
    all_rows[#all_rows + 1] = row
  end

  up = merge_up(all_rows)

  return tbl
end
