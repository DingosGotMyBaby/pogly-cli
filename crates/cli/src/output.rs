use serde_json::Value;

pub fn print_json(value: &Value) {
    println!(
        "{}",
        serde_json::to_string_pretty(value).unwrap_or_default()
    );
}

pub fn table(headers: &[&str], rows: &[Vec<String>]) {
    let mut widths: Vec<usize> = headers.iter().map(|h| h.chars().count()).collect();
    for row in rows {
        for (i, cell) in row.iter().enumerate() {
            if i < widths.len() {
                widths[i] = widths[i].max(cell.chars().count());
            }
        }
    }
    let print_row = |cells: Vec<&str>| {
        let line = cells
            .iter()
            .enumerate()
            .map(|(i, c)| format!("{:<width$}", c, width = widths[i]))
            .collect::<Vec<_>>()
            .join("  ");
        println!("{}", line.trim_end());
    };
    print_row(headers.to_vec());
    for row in rows {
        print_row(row.iter().map(String::as_str).collect());
    }
}

pub fn truncate(value: &str, max: usize) -> String {
    let clean: String = value
        .chars()
        .map(|c| if c == '\n' || c == '\r' { ' ' } else { c })
        .collect();
    if clean.chars().count() <= max {
        clean
    } else {
        let cut: String = clean.chars().take(max.saturating_sub(1)).collect();
        format!("{cut}…")
    }
}
