use serde::Serialize;
use std::cmp::Ordering;

/// Utility functions for handling output formatting and file operations
pub struct OutputHandler;

impl OutputHandler {
    /// Write data to JSON file if path is provided, otherwise print to stdout
    #[allow(dead_code)]
    pub fn handle_json_output<T: Serialize>(
        data: &T,
        output_path: &Option<String>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        match output_path {
            Some(path) => {
                let json = serde_json::to_string_pretty(data)?;
                std::fs::write(path, json)?;
            }
            None => {
                let json = serde_json::to_string_pretty(data)?;
                println!("{json}");
            }
        }
        Ok(())
    }

    /// Print a limited number of items from a collection
    pub fn print_limited<T: std::fmt::Debug>(items: &[T], limit: Option<usize>) {
        let limit = limit.unwrap_or(items.len());
        for item in items.iter().take(limit) {
            println!("{item:?}");
        }
    }
}

/// Sorting utilities for different data types
pub struct SortingUtils;

impl SortingUtils {
    /// Sort entanglement sets by entanglement value
    pub fn sort_entanglement_sets<T>(sets: &mut [T])
    where
        T: HasEntanglement,
    {
        sets.sort_by_key(|a| a.get_entanglement());
    }

    /// Sort entropy sets by entropy value
    pub fn sort_entropy_sets<T>(sets: &mut [T])
    where
        T: HasEntropy,
    {
        sets.sort_by(|a, b| {
            a.get_entropy()
                .partial_cmp(&b.get_entropy())
                .unwrap_or(Ordering::Equal)
        });
    }
}

/// Trait for types that have an entanglement value
pub trait HasEntanglement {
    fn get_entanglement(&self) -> usize;
}

/// Trait for types that have an entropy value
pub trait HasEntropy {
    fn get_entropy(&self) -> f32;
}

/// Statistics calculation utilities
pub struct StatsUtils;

impl StatsUtils {
    /// Calculate min and max entanglement from a collection
    pub fn entanglement_min_max<T>(items: &[T]) -> Option<(usize, usize)>
    where
        T: HasEntanglement,
    {
        if items.is_empty() {
            return None;
        }

        let mut min = items[0].get_entanglement();
        let mut max = min;

        for item in items.iter().skip(1) {
            let val = item.get_entanglement();
            if val < min {
                min = val;
            }
            if val > max {
                max = val;
            }
        }

        Some((min, max))
    }

    /// Calculate min and max entropy from a collection
    pub fn entropy_min_max<T>(items: &[T]) -> Option<(f64, f64)>
    where
        T: HasEntropy,
    {
        if items.is_empty() {
            return None;
        }

        let mut min = items[0].get_entropy();
        let mut max = min;

        for item in items.iter().skip(1) {
            let val = item.get_entropy();
            if val < min {
                min = val;
            }
            if val > max {
                max = val;
            }
        }

        Some((min as f64, max as f64))
    }
}
