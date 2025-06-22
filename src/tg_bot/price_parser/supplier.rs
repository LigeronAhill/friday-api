use std::{fmt::Display, io::Cursor};

use anyhow::{anyhow, Error, Result};
use bytes::Bytes;
use calamine::{Data, Range, Reader, Sheets};
pub enum Supplier {
    Fancy,
    Fox,
}

impl TryFrom<Sheets<Cursor<Bytes>>> for Supplier {
    type Error = Error;

    fn try_from(value: Sheets<Cursor<Bytes>>) -> Result<Self> {
        let mut wb = value;
        let sheets = wb.worksheets();
        let (first_sheet_name, first_sheet) =
            sheets.first().ok_or(anyhow!("No sheets"))?.to_owned();
        if first_sheet_name.to_lowercase() == "обновления" {
            return Ok(Supplier::Fox);
        } else if is_fancy(first_sheet) {
            return Ok(Supplier::Fancy);
        }
        Err(anyhow!("Неизвестный поставщик"))
    }
}

impl Display for Supplier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Supplier::Fancy => write!(f, "ФЭНСИ ФЛОР"),
            Supplier::Fox => write!(f, "БРАТЕЦ ЛИС"),
        }
    }
}

fn is_fancy(table: Range<Data>) -> bool {
    for (i, row) in table.rows().enumerate() {
        if i < 7 {
            for cell in row {
                if cell.to_string().to_lowercase().contains("фэнси флор") {
                    return true;
                }
            }
        }
    }
    false
}
