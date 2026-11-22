from openpyxl import Workbook
from openpyxl.styles import Font, PatternFill, Alignment, Border, Side
from openpyxl.utils import get_column_letter as L
from openpyxl.worksheet.datavalidation import DataValidation
from openpyxl.comments import Comment



def build(T, path):
    wb = Workbook()
    X = T
    F = lambda **k: Font(name='Arial', **k)
    GREEN = '1D5A44'
    thin = Side(style='thin', color='C9CFC9')
    BOX = Border(left=thin, right=thin, top=thin, bottom=thin)
    YEL = PatternFill('solid', fgColor='FFF2B3')
    SAND = PatternFill('solid', fgColor='F1E7D6')
    SAGE = PatternFill('solid', fgColor='DFE9E2')
    HEAD = PatternFill('solid', fgColor=GREEN)

    # ---------- Instrukcja ----------
    ins = wb.active
    ins.title = X['sh_ins']
    ins.column_dimensions['A'].width = 3
    ins.column_dimensions['B'].width = 100
    rows = [(X['ins'][0], F(size=16, bold=True, color=GREEN)), ('', None), (X['ins'][1], F(size=12, bold=True))] + [(t, F()) for t in X['ins'][2:7]] + [
        ('', None), (X['ins'][7], F(size=12, bold=True)), (X['ins'][8], F()), ('', None), (X['ins'][9], F(size=12, bold=True)), (X['ins'][10], F())]
    for i, (t, f) in enumerate(rows, start=2):
        ins.cell(row=i, column=2, value=t)
        if f: ins.cell(row=i, column=2).font = f
        ins.cell(row=i, column=2).alignment = Alignment(wrap_text=True, vertical='top')

    # ---------- Osoby ----------
    os_ = wb.create_sheet(X['sh_people'])
    cols = list(zip(X['cols'], [6, 14, 16, 16, 10, 14, 18, 14, 18, 9, 9, 18, 40]))
    for c, (h, w) in enumerate(cols, start=1):
        cell = os_.cell(row=1, column=c, value=h)
        cell.font = F(bold=True, color='FFFFFF')
        cell.fill = HEAD
        cell.alignment = Alignment(vertical='center', wrap_text=True)
        os_.column_dimensions[L(c)].width = w
    os_.row_dimensions[1].height = 30
    G = X['g']
    occ = X['occ']
    example = [
        (1, 'Zofia', 'Zawadzka', '', G[1], '2012-05-04', 'Wrocław', '', '', 2, 3, '', X['ex_note']),
        (2, 'Michał', 'Zawadzki', '', G[0], '1980-10-15', 'Wrocław', '', '', 4, 5, occ[0], ''),
        (3, 'Julia', 'Zawadzka', 'Krawczyk', G[1], '1985-03-08', 'Łódź', '', '', '', '', occ[1], ''),
        (4, 'Andrzej', 'Zawadzki', '', G[0], '1952-08-08', 'Wrocław', '', '', 6, 7, occ[2], ''),
        (5, 'Katarzyna', 'Zawadzka', 'Sobczak', G[1], '1956-02-11', 'Poznań', '', '', '', '', occ[3], ''),
        (6, 'Tadeusz', 'Zawadzki', '', G[0], '1923-03-14', 'Kraków', '2004-11-02', 'Wrocław', '', '', occ[4], ''),
        (7, 'Barbara', 'Zawadzka', 'Nowicka', G[1], '1928-06-21', 'Lwów', '2012-04-04', 'Wrocław', '', '', occ[5], ''),
    ]
    for r, row in enumerate(example, start=2):
        for c, v in enumerate(row, start=1):
            cell = os_.cell(row=r, column=c, value=v if v != '' else None)
            cell.font = F(color='6B746E', italic=True)
    for r in range(2, 502):
        for c in range(1, 14):
            cell = os_.cell(row=r, column=c)
            cell.border = BOX
            if c in (2, 3, 4, 6, 7, 8, 9, 12, 13):
                cell.number_format = '@'
            if r > 8:
                cell.fill = YEL
                cell.font = F()
    for r in range(2, 9):
        for c in range(1, 14):
            os_.cell(row=r, column=c).fill = YEL
    dv = DataValidation(type='list', formula1='"' + G[0] + ',' + G[1] + '"', allow_blank=True)
    os_.add_data_validation(dv)
    dv.add('E2:E501')
    os_.freeze_panes = 'B2'
    os_['A1'].comment = Comment(X['id_comment'], 'myroots.cc')
    os_.auto_filter.ref = 'A1:M501'

    # ---------- Drzewo 4 pokolenia ----------
    tr = wb.create_sheet(X['sh_tree'])
    tr.sheet_view.showGridLines = False
    for c in range(1, 18):
        tr.column_dimensions[L(c)].width = 3 if c == 1 else 11.5
    tr['B2'] = X['tree_h']
    tr['B2'].font = F(size=16, bold=True, color=GREEN)
    tr['B3'] = X['tree_id']
    tr['B3'].font = F(bold=True)
    tr['E3'] = 1
    tr['E3'].fill = YEL
    tr['E3'].font = F(bold=True, size=12)
    tr['E3'].border = BOX
    tr['E3'].alignment = Alignment(horizontal='center')
    tr['F3'] = X['tree_hint']
    tr['F3'].font = F(color='6B746E', italic=True)

    SH = "'" + X['sh_people'].replace("'", "''") + "'!"
    R = SH + "$A$2:$A$501"
    def col(letter): return f"{SH}${letter}$2:${letter}$501"
    def look(idref, letter):
        return f'INDEX({col(letter)},MATCH({idref},{R},0))'

    # generations: (label, count, id row, name row, years row, fill)
    gens = [(X['gens'][0], 8, 5, 6, 7, SAND), (X['gens'][1], 4, 9, 10, 11, SAGE), (X['gens'][2], 2, 13, 14, 15, SAND), (X['gens'][3], 1, 17, 18, 19, SAGE)]
    width = 16  # columns B..Q
    for gi, (label, n, idr, nr, yr, fill) in enumerate(gens):
        span = width // n
        tr.row_dimensions[idr].hidden = True
        tr.row_dimensions[nr].height = 30
        tr.row_dimensions[yr].height = 18
        for i in range(n):
            c0 = 2 + i * span
            c1 = c0 + span - 1
            idc = f'{L(c0)}{idr}'
            # ID of this person
            if gi == 3:
                tr[idc] = '=IF($E$3="","",$E$3)'
            else:
                child_n = n // 2
                child_span = width // child_n
                child_col = 2 + (i // 2) * child_span
                child_id = f'{L(child_col)}{gens[gi + 1][2]}'
                parent_col = 'J' if i % 2 == 0 else 'K'
                tr[idc] = f'=IF({child_id}="","",IFERROR(IF({look(child_id, parent_col)}="","",{look(child_id, parent_col)}),""))'
            # name and years
            tr.merge_cells(start_row=nr, start_column=c0, end_row=nr, end_column=c1)
            tr.merge_cells(start_row=yr, start_column=c0, end_row=yr, end_column=c1)
            name = tr.cell(row=nr, column=c0)
            name.value = f'=IF({idc}="","",IFERROR(TRIM({look(idc, "B")}&" "&{look(idc, "C")}),""))'
            name.font = F(bold=True, size=10 if n == 8 else 11)
            name.alignment = Alignment(horizontal='center', vertical='center', wrap_text=True)
            yrs = tr.cell(row=yr, column=c0)
            yrs.value = (f'=IF({idc}="","",IFERROR(IF({look(idc, "H")}="",IF({look(idc, "F")}="","","' + X['born'] + f' "&{look(idc, "F")}),'
                         f'{look(idc, "F")}&" – "&{look(idc, "H")}),""))')
            yrs.font = F(size=8, color='6B746E')
            yrs.alignment = Alignment(horizontal='center', vertical='top')
            for r_ in (nr, yr):
                for cc in range(c0, c1 + 1):
                    cell = tr.cell(row=r_, column=cc)
                    cell.fill = fill
                    cell.border = Border(left=thin if cc == c0 else None, right=thin if cc == c1 else None,
                                         top=thin if r_ == nr else None, bottom=thin if r_ == yr else None)
        tr.cell(row=nr, column=18, value=label).font = F(color='6B746E', italic=True, size=9)
    tr.column_dimensions['R'].width = 14
    tr['B21'] = X['foot']
    tr['B21'].font = F(size=9, color='6B746E', italic=True)
    tr.page_setup.orientation = 'landscape'
    tr.page_setup.fitToWidth = 1
    tr.sheet_properties.pageSetUpPr.fitToPage = True
    wb.active = 2
    if X.get('rtl'):
        for ws in wb.worksheets: ws.sheet_view.rightToLeft = True
    wb.save(path)

