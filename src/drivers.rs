use prettytable::{cell, row, Table};

pub trait Driver {
    fn print_info(&self, name: &str);
}

pub fn name_desc_to_table(parts: &[(String, String)]) -> Table {
    let mut table = Table::new();
    table.set_titles(row![bFg->"Name", bFg->"Description"]);
    for part in parts {
        table.add_row(row![Fc->&part.0, &part.1]);
    }
    table
}
