use tabled::{Table, Tabled};
use tabled::settings::{Style, Width, object::Columns, Modify};

#[derive(Tabled)]
pub struct PropertyTableRow {
    #[tabled(rename = "Title")]
    pub title: String,
    #[tabled(rename = "History", display_with = "display_right_12")]
    pub price_history: String,
    #[tabled(rename = "Size (m²)", display_with = "display_right_8")]
    pub size: String,
    #[tabled(rename = "Rooms", display_with = "display_right_5")]
    pub rooms: String,
    #[tabled(rename = "Address")]
    pub address: String,
}

fn display_right_12(s: &str) -> String {
    format!("{:>12}", s)
}

fn display_right_5(s: &str) -> String {
    format!("{:>5}", s)
}

fn display_right_8(s: &str) -> String {
    format!("{:>8}", s)
}

impl PropertyTableRow {
    pub fn from_property_display(display: &crate::PropertyDisplay, graph_height: u8) -> Self {
        let history = display.price_history.as_ref()
            .map(|h| h.to_ascii_graph(12, graph_height as usize).replace('\n', " "))
            .unwrap_or_else(|| "No hist".to_string());

        let size_str = display.property.covered_size
            .map(|s| format!("{}m²", s.round() as i64))
            .unwrap_or_else(|| "N/A".to_string());

        let rooms_str = display.property.rooms
            .map(|r| r.to_string())
            .unwrap_or_else(|| "N/A".to_string());

        Self {
            title: display.property.title.clone(),
            price_history: history,
            size: size_str,
            rooms: rooms_str,
            address: display.property.address.clone(),
        }
    }
}

pub fn create_property_table(displays: &[crate::PropertyDisplay], graph_height: u8) -> String {
    let table_rows: Vec<PropertyTableRow> = displays.iter()
        .map(|d| PropertyTableRow::from_property_display(d, graph_height))
        .collect();

    let mut table = Table::new(&table_rows);
    
    // Configure table style and column widths
    table
        .with(Style::modern())
        .with(Modify::new(Columns::single(0)).with(Width::truncate(40)))     // Title column
        .with(Modify::new(Columns::single(1)).with(Width::truncate(12)))     // History column
        .with(Modify::new(Columns::single(2)).with(Width::truncate(8)))      // Size column
        .with(Modify::new(Columns::single(3)).with(Width::truncate(5)))      // Rooms column
        .with(Modify::new(Columns::single(4)).with(Width::wrap(60)));        // Address column

    table.to_string()
} 