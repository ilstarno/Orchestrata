const invalid = () => ({ ok: false, issues: [{ code: 'invalid-cell-edit' }] });
export function updateSpreadsheetCell(workbook, sheetIndex, sourceRow, column, value) {
    const sheet = workbook.sheets[sheetIndex];
    if (!sheet?.sourceRows || !Number.isInteger(sheetIndex) || sheetIndex < 0
        || !Number.isInteger(sourceRow) || sourceRow <= (sheet.headingRow ?? 1)
        || !Number.isInteger(column) || column < 0 || column >= sheet.headers.length
        || typeof value !== 'string' || value.length > 8192 || /[\u0000-\u001f\u007f]/u.test(value))
        return invalid();
    const modelIndex = sheet.rowNumbers.indexOf(sourceRow);
    const sourceIndex = sourceRow - 1;
    const model = sheet.rows[modelIndex];
    const physical = sheet.sourceRows[sourceIndex];
    if (!model || !physical || (model[column] ?? null) !== (physical[column] ?? null))
        return invalid();
    const nextModel = model.slice();
    const nextPhysical = physical.slice();
    nextModel[column] = value;
    nextPhysical[column] = value;
    if (!nextModel.some(cell => cell != null && String(cell).trim() !== '')) {
        return { ok: false, issues: [{ code: 'empty-source-row' }] };
    }
    const rows = sheet.rows.slice();
    const sourceRows = sheet.sourceRows.slice();
    rows[modelIndex] = nextModel;
    sourceRows[sourceIndex] = nextPhysical;
    const sheets = workbook.sheets.slice();
    sheets[sheetIndex] = { ...sheet, rows, sourceRows };
    return { ok: true, workbook: { sheets } };
}
