use serde::{Deserialize, Serialize};
use std::fmt;
use std::ops::{Add, Div, Mul, Sub};

pub const POINTS_PER_INCH: f64 = 72.0;
pub const MM_PER_INCH: f64 = 25.4;
pub const CM_PER_INCH: f64 = 2.54;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Default)]
pub enum Unit {
    #[default]
    Point,      // pt (1/72 inch)
    Millimeter, // mm
    Centimeter, // cm
    Inch,       // in
    Pixel,      // px (at document DPI)
}

impl fmt::Display for Unit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Unit::Point => write!(f, "pt"),
            Unit::Millimeter => write!(f, "mm"),
            Unit::Centimeter => write!(f, "cm"),
            Unit::Inch => write!(f, "in"),
            Unit::Pixel => write!(f, "px"),
        }
    }
}

impl Unit {
    /// Converts a value from this unit to absolute points (pt).
    /// For Unit::Pixel, the document's DPI must be provided.
    pub fn to_points(self, value: f64, dpi: Option<u32>) -> Pt {
        let pts = match self {
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
        };
        Pt(pts)
    }

    /// Converts absolute points (pt) to this unit.
    pub fn from_points(self, points: Pt, dpi: Option<u32>) -> f64 {
        match self {
            Unit::Point => points.0,
            Unit::Millimeter => points.0 * (MM_PER_INCH / POINTS_PER_INCH),
            Unit::Centimeter => points.0 * (CM_PER_INCH / POINTS_PER_INCH),
            Unit::Inch => points.0 / POINTS_PER_INCH,
            Unit::Pixel => {
                let d = dpi.expect("DPI required for pixel conversion") as f64;
                if d <= 0.0 {
                    panic!("DPI must be greater than 0 for pixel conversion");
                }
                points.0 * (d / POINTS_PER_INCH)
            }
        }
    }
}

/// Typed unit wrapper for points (pt).
/// Internal representation of all dimensions in the document.
/// 1 pt = 1/72 inch.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, PartialOrd)]
#[serde(transparent)]
pub struct Pt(pub f64);

impl Pt {
    pub fn new(value: f64) -> Self {
        Pt(value)
    }

    pub fn from_unit(unit: Unit, value: f64, dpi: Option<u32>) -> Self {
        unit.to_points(value, dpi)
    }

    pub fn to_unit(self, unit: Unit, dpi: Option<u32>) -> f64 {
        unit.from_points(self, dpi)
    }

    pub fn value(self) -> f64 {
        self.0
    }
}

impl Default for Pt {
    fn default() -> Self {
        Pt(0.0)
    }
}

impl fmt::Display for Pt {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}pt", self.0)
    }
}

impl From<f64> for Pt {
    fn from(value: f64) -> Self {
        Pt(value)
    }
}

impl From<Pt> for f64 {
    fn from(pt: Pt) -> Self {
        pt.0
    }
}

impl Add for Pt {
    type Output = Pt;

    fn add(self, other: Pt) -> Pt {
        Pt(self.0 + other.0)
    }
}

impl Sub for Pt {
    type Output = Pt;

    fn sub(self, other: Pt) -> Pt {
        Pt(self.0 - other.0)
    }
}

impl Mul<f64> for Pt {
    type Output = Pt;

    fn mul(self, scalar: f64) -> Pt {
        Pt(self.0 * scalar)
    }
}

impl Mul<Pt> for f64 {
    type Output = Pt;

    fn mul(self, pt: Pt) -> Pt {
        Pt(self * pt.0)
    }
}

impl Div<f64> for Pt {
    type Output = Pt;

    fn div(self, scalar: f64) -> Pt {
        Pt(self.0 / scalar)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_unit_conversions() {
        let dpi = Some(300);

        // 1 inch = 72 pt
        assert_eq!(Unit::Inch.to_points(1.0, None).0, 72.0);
        assert_eq!(Unit::Inch.from_points(Pt(72.0), None), 1.0);

        // 25.4 mm = 1 inch = 72 pt
        assert!((Unit::Millimeter.to_points(25.4, None).0 - 72.0).abs() < 0.0001);
        assert!((Unit::Millimeter.from_points(Pt(72.0), None) - 25.4).abs() < 0.0001);

        // Pixel at 300 DPI: 300 px = 1 inch = 72 pt
        assert!((Unit::Pixel.to_points(300.0, dpi).0 - 72.0).abs() < 0.0001);
        assert!((Unit::Pixel.from_points(Pt(72.0), dpi) - 300.0).abs() < 0.0001);
    }

    #[test]
    fn test_round_trip_millimeter() {
        let dpi = None;
        let original_mm = 25.4;

        // mm -> pt -> mm
        let pts = Unit::Millimeter.to_points(original_mm, dpi);
        let back_to_mm = Unit::Millimeter.from_points(pts, dpi);

        assert!((back_to_mm - original_mm).abs() < 0.001);
    }

    #[test]
    fn test_round_trip_centimeter() {
        let dpi = None;
        let original_cm = 2.54;

        // cm -> pt -> cm
        let pts = Unit::Centimeter.to_points(original_cm, dpi);
        let back_to_cm = Unit::Centimeter.from_points(pts, dpi);

        assert!((back_to_cm - original_cm).abs() < 0.001);
    }

    #[test]
    fn test_round_trip_inch() {
        let dpi = None;
        let original_in = 1.0;

        // in -> pt -> in
        let pts = Unit::Inch.to_points(original_in, dpi);
        let back_to_in = Unit::Inch.from_points(pts, dpi);

        assert!((back_to_in - original_in).abs() < 0.001);
    }

    #[test]
    fn test_round_trip_pixel() {
        let dpi = Some(300);
        let original_px = 150.0;

        // px -> pt -> px
        let pts = Unit::Pixel.to_points(original_px, dpi);
        let back_to_px = Unit::Pixel.from_points(pts, dpi);

        assert!((back_to_px - original_px).abs() < 0.001);
    }

    #[test]
    fn test_round_trip_multiple_units() {
        let test_value = 100.0; // mm

        // mm -> cm -> mm
        let mm_to_cm =
            Unit::Millimeter.from_points(Unit::Millimeter.to_points(test_value, None), None);
        assert!((mm_to_cm - test_value).abs() < 0.001);

        // mm -> in -> mm
        let mm_to_in = Unit::Millimeter.from_points(
            Unit::Inch.to_points(
                Unit::Inch.from_points(Unit::Millimeter.to_points(test_value, None), None),
                None,
            ),
            None,
        );
        assert!((mm_to_in - test_value).abs() < 0.001);
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

    #[test]
    fn test_pt_arithmetic() {
        let a = Pt(10.0);
        let b = Pt(5.0);

        assert_eq!((a + b).0, 15.0);
        assert_eq!((a - b).0, 5.0);
        assert_eq!((a * 2.0).0, 20.0);
        assert_eq!((2.0 * b).0, 10.0);
        assert_eq!((a / 2.0).0, 5.0);
    }

    #[test]
    fn test_pt_conversions() {
        let pt = Pt(72.0);

        let mm = Unit::Millimeter.from_points(pt, None);
        assert!((mm - 25.4).abs() < 0.001);

        let in_val = Unit::Inch.from_points(pt, None);
        assert_eq!(in_val, 1.0);
    }
}
