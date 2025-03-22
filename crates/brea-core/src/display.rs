use tabled::{Table, Tabled};
use tabled::settings::{Style, Width, object::Columns, Modify};

#[derive(Tabled)]
pub struct PropertyTableRow {
    #[tabled(rename = "Title")]
    pub title: String,
    #[tabled(rename = "History", display_with = "display_right_8")]
    pub price_history: String,
    #[tabled(rename = "Price (kUSD)", display_with = "display_right_8")]
    pub price: String,
    #[tabled(rename = "Rooms", display_with = "display_right_5")]
    pub rooms: String,
    #[tabled(rename = "Size (m²)", display_with = "display_right_8")]
    pub size: String,
    #[tabled(rename = "Address")]
    pub address: String,
}

fn display_right_8(s: &str) -> String {
    format!("{:>8}", s)
}

fn display_right_5(s: &str) -> String {
    format!("{:>5}", s)
}

impl PropertyTableRow {
    pub fn from_property_display(display: &crate::PropertyDisplay, graph_height: u8) -> Self {
        let history = display.price_history.as_ref()
            .map(|h| h.to_ascii_graph(8, graph_height as usize).replace('\n', " "))
            .unwrap_or_else(|| "No hist".to_string());

        Self {
            title: display.property.title.clone(),
            price_history: history,
            price: format!("{}k", (display.property.price_usd / 1000.0).round() as i64),
            rooms: display.property.rooms.to_string(),
            size: format!("{}m²", display.property.covered_size.round() as i64),
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
        .with(Modify::new(Columns::single(1)).with(Width::truncate(8)))      // History column
        .with(Modify::new(Columns::single(2)).with(Width::truncate(10)))     // Price column
        .with(Modify::new(Columns::single(3)).with(Width::truncate(5)))      // Rooms column
        .with(Modify::new(Columns::single(4)).with(Width::truncate(8)))      // Size column
        .with(Modify::new(Columns::single(5)).with(Width::wrap(60)));        // Address column

    table.to_string()
} 