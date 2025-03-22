use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};
use rasciigraph::{plot, Config};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriceHistory {
    pub prices: Vec<(f64, DateTime<Utc>)>,
}

impl PriceHistory {
    pub fn new(prices: Vec<(f64, DateTime<Utc>)>) -> Self {
        Self { 
            prices: prices.into_iter().rev().collect(), // Reverse to get chronological order
        }
    }

    pub fn to_ascii_graph(&self, width: usize, height: usize) -> String {
        if self.prices.is_empty() {
            return "No hist".to_string();
        }

        // Extract prices and convert to kUSD, rounding to whole numbers
        let prices: Vec<f64> = self.prices.iter()
            .map(|(price, _)| (*price / 1000.0).round())
            .collect();

        // Create the graph using rasciigraph
        let config = Config::default()
            .with_width(width as u32)
            .with_height(height as u32);
        let graph = plot(prices, config);

        // Ensure each line is exactly the specified width
        graph.lines()
            .map(|line| format!("{:width$}", line, width = width))
            .collect::<Vec<_>>()
            .join("\n")
    }
} 