use serde::{Deserialize, Serialize};

pub const POINTS_PER_INCH: f64 = 72.0;
pub const MM_PER_INCH: f64 = 25.4;
pub const CM_PER_INCH: f64 = 2.54;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum Unit {
    Point,      // pt (1/72 inch)
    Millimeter, // mm
    Centimeter, // cm
    Inch,       // in
    Pixel,      // px (at document DPI)
}

impl Default for Unit {
    fn default() -> Self {
        Unit::Point
    }
}

impl Unit {
    /// Converts a value from this unit to absolute points (pt).
    /// For Unit::Pixel, the document's DPI must be provided.
    pub fn to_points(self, value: f64, dpi: Option<u32>) -> f64 {
        match self {
            Unit::Point => value,
            Unit::Millimeter => value * (POINTS_PER_INCH / MM_PER_INCH),
            Unit::Centimeter => value * (POINTS_PER_INCH / CM_PER_INCH),
            Unit::Inch => value * POINTS_PER_INCH,
            Unit::Pixel => {
                let d = dpi.expect("DPI required for pixel conversion") as f64;
                if d <= 0.0 {
                    panic!("DPI must be greater than 0 for pixel conversion");
                }
                value * (POINTS_PER_INCH / d)
            }
        }
    }

    /// Converts absolute points (pt) to this unit.
    pub fn from_points(self, points: f64, dpi: Option<u32>) -> f64 {
        match self {
            Unit::Point => points,
            Unit::Millimeter => points * (MM_PER_INCH / POINTS_PER_INCH),
            Unit::Centimeter => points * (CM_PER_INCH / POINTS_PER_INCH),
            Unit::Inch => points / POINTS_PER_INCH,
            Unit::Pixel => {
                let d = dpi.expect("DPI required for pixel conversion") as f64;
                if d <= 0.0 {
                    panic!("DPI must be greater than 0 for pixel conversion");
                }
                points * (d / POINTS_PER_INCH)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_unit_conversions() {
        let dpi = Some(300);
        
        // 1 inch = 72 pt
        assert_eq!(Unit::Inch.to_points(1.0, None), 72.0);
        assert_eq!(Unit::Inch.from_points(72.0, None), 1.0);
        
        // 25.4 mm = 1 inch = 72 pt
        assert!((Unit::Millimeter.to_points(25.4, None) - 72.0).abs() < 0.0001);
        assert!((Unit::Millimeter.from_points(72.0, None) - 25.4).abs() < 0.0001);
        
        // Pixel at 300 DPI: 300 px = 1 inch = 72 pt
        assert!((Unit::Pixel.to_points(300.0, dpi) - 72.0).abs() < 0.0001);
        assert!((Unit::Pixel.from_points(72.0, dpi) - 300.0).abs() < 0.0001);
    }
    
    #[test]
    #[should_panic(expected = "DPI required")]
    fn test_pixel_requires_dpi() {
        Unit::Pixel.to_points(100.0, None);
    }

    #[test]
    #[should_panic(expected = "DPI must be greater than 0")]
    fn test_pixel_requires_positive_dpi() {
        Unit::Pixel.to_points(100.0, Some(0));
    }
}
