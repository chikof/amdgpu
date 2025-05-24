//! This module provides functionality to format various units of measurement, such as temperature,
//! memory, GPU usage, power, and frequency.

/// Units of measurement for formatting.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum Units {
    Temperature, // °C
    Memory,      // bytes → KB/MB/…
    Gpu,         // 0.0–1.0 → %
    Power,       // W
    Frequency,   // Hz → kHz/MHz/…
}

/// Generic “scale-and-suffix” formatter
fn format_scaled(value: f64, suffixes: &[(&str, f64)]) -> String {
    for &(suffix, threshold) in suffixes {
        if value >= threshold {
            return format!("{:.2} {}", value / threshold, suffix);
        }
    }
    format!("{:.0}", value)
}

const BYTE_SUFFIXES: [(&str, f64); 5] = [
    ("TB", 1_099_511_627_776.0), // 1024^4
    ("GB", 1_073_741_824.0),     // 1024^3
    ("MB", 1_048_576.0),         // 1024^2
    ("KB", 1_024.0),             // 1024^1
    ("B", 1.0),
];

const HERTZ_SUFFIXES: [(&str, f64); 4] = [
    ("GHz", 1_000_000_000.0),
    ("MHz", 1_000_000.0),
    ("kHz", 1_000.0),
    ("Hz", 1.0),
];

/// Formats a value in the specified unit, returning a string with the appropriate suffix.
pub fn format_units(unit: Units, value: f64) -> String {
    match unit {
        Units::Temperature => format!("{:.1} °C", value),
        Units::Gpu => format!("{:.1}%", value * 100.0),
        Units::Power => format!("{:.1} W", value),
        Units::Memory => format_scaled(value, &BYTE_SUFFIXES),
        Units::Frequency => format_scaled(value, &HERTZ_SUFFIXES),
    }
}
