use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum Unit {
    Point,      // pt (1/72 inch)
    Millimeter, // mm
    Centimeter, // cm
    Inch,       // in
    Pixel,      // px (at document DPI)
}

impl Unit {
    pub fn to_points(self, value: f64, dpi: u32) -> f64 {
        match self {
            Unit::Point => value,
            Unit::Millimeter => value * (72.0 / 25.4),
            Unit::Centimeter => value * (72.0 / 2.54),
            Unit::Inch => value * 72.0,
            Unit::Pixel => value * (72.0 / dpi as f64),
        }
    }

    pub fn from_points(self, points: f64, dpi: u32) -> f64 {
        match self {
            Unit::Point => points,
            Unit::Millimeter => points * (25.4 / 72.0),
            Unit::Centimeter => points * (2.54 / 72.0),
            Unit::Inch => points / 72.0,
            Unit::Pixel => points * (dpi as f64 / 72.0),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_unit_conversions() {
        let dpi = 300;
        
        // 1 inch = 72 pt
        assert_eq!(Unit::Inch.to_points(1.0, dpi), 72.0);
        assert_eq!(Unit::Inch.from_points(72.0, dpi), 1.0);
        
        // 25.4 mm = 1 inch = 72 pt
        assert!((Unit::Millimeter.to_points(25.4, dpi) - 72.0).abs() < 0.0001);
        assert!((Unit::Millimeter.from_points(72.0, dpi) - 25.4).abs() < 0.0001);
        
        // Pixel at 300 DPI: 300 px = 1 inch = 72 pt
        assert_eq!(Unit::Pixel.to_points(300.0, dpi), 72.0);
        assert_eq!(Unit::Pixel.from_points(72.0, dpi), 300.0);
    }
}
