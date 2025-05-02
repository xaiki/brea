use chrono::{DateTime, Utc};
use colored::Colorize;
use std::fmt;

pub struct PropertyDisplay {
    pub property: super::Property,
    pub price_history: Vec<(f64, DateTime<Utc>)>,
}

impl fmt::Display for PropertyDisplay {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

impl PropertyDisplay {
    pub fn new(property: super::Property, price_history: Vec<(f64, DateTime<Utc>)>) -> Self {
        Self {
            property,
            price_history,
        }
    }

    fn create_ascii_graph(data: &[(f64, DateTime<Utc>)], width: usize, height: usize) -> String {
        if data.is_empty() {
            return String::new();
        }

        let mut graph = vec![vec![' '; width]; height];
        let max_price = data.iter().map(|(p, _)| *p).fold(f64::NEG_INFINITY, f64::max);
        let min_price = data.iter().map(|(p, _)| *p).fold(f64::INFINITY, f64::min);
        let price_range = max_price - min_price;

        for (i, (price, _)) in data.iter().enumerate() {
            if i >= width {
                break;
            }
            let normalized_height = if price_range > 0.0 {
                ((price - min_price) / price_range * (height as f64 - 1.0)) as usize
            } else {
                height / 2
            };
            for y in 0..=normalized_height {
                graph[y][i] = '█';
            }
        }

        graph.into_iter()
            .rev()
            .map(|row| row.into_iter().collect::<String>())
            .collect::<Vec<_>>()
            .join("\n")
    }

    pub fn to_string(&self) -> String {
        let graph = Self::create_ascii_graph(&self.price_history, 40, 10);
        
        let mut details = Vec::new();
        
        if let Some(size) = self.property.covered_size {
            details.push(format!("{:.1} m²", size));
        }
        
        if let Some(rooms) = self.property.rooms {
            details.push(format!("{} rooms", rooms));
        }
        
        if let Some(antiquity) = self.property.antiquity {
            details.push(format!("{} years old", antiquity));
        }
        
        let details_str = details.join(" | ");
        let price_str = format!("${:.2}", self.property.price_usd);
        
        format!(
            "{}\n{}\n{}\n{}\n{}\n{}",
            self.property.title.bright_white().bold(),
            self.property.description.as_deref().unwrap_or(""),
            format!("Price: {}", price_str).green(),
            if !details_str.is_empty() {
                format!("Details: {}", details_str)
            } else {
                String::new()
            },
            format!("Address: {} - {}", self.property.address, self.property.district),
            if !graph.is_empty() {
                format!("\nPrice History:\n{}", graph)
            } else {
                String::new()
            }
        )
    }
} 