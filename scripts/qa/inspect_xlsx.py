#!/usr/bin/env python3
"""Dump an exported plan workbook to JSON: sheets, dimensions, merged ranges, page setup, freeze panes, all cell values (first N rows)."""
import sys, json, openpyxl
f, out = sys.argv[1], sys.argv[2]
wb = openpyxl.load_workbook(f)  # formulas as written
d = {'file': f, 'sheets': []}
for ws in wb:
    ps = ws.page_setup
    rows = [[c if not hasattr(c, 'isoformat') else c.isoformat() for c in r] for r in ws.iter_rows(values_only=True)]
    d['sheets'].append(dict(title=ws.title, dims=ws.dimensions, max_row=ws.max_row, max_col=ws.max_column,
        merged=[str(m) for m in ws.merged_cells.ranges], freeze=ws.freeze_panes,
        orientation=ps.orientation, paper_size=ps.paperSize, fit_to_width=ps.fitToWidth, fit_to_height=ps.fitToHeight,
        fit_to_page=bool(ws.sheet_properties.pageSetUpPr and ws.sheet_properties.pageSetUpPr.fitToPage),
        print_area=ws.print_area, print_titles=ws.print_title_rows, header=ws.oddHeader.center.text if ws.oddHeader else None, footer=ws.oddFooter.center.text if ws.oddFooter else None,
        rows=rows[:200]))
json.dump(d, open(out, 'w'), ensure_ascii=False, indent=1)
print([ (s['title'], s['dims'], len(s['merged'])) for s in d['sheets']])
