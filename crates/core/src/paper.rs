use crate::units::Unit;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PaperFormat {
    A0, A1, A2, A3, A4, A5, A6,
    B0, B1, B2, B3, B4, B5, B6,
    Letter,
    Legal,
    Custom { width_pt: f64, height_pt: f64 },
}

impl PaperFormat {
    pub fn dimensions_pt(self) -> (f64, f64) {
        match self {
            // A-series (based on mm, converted to pt)
            PaperFormat::A0 => (Unit::Millimeter.to_points(841.0, 72), Unit::Millimeter.to_points(1189.0, 72)),
            PaperFormat::A1 => (Unit::Millimeter.to_points(594.0, 72), Unit::Millimeter.to_points(841.0, 72)),
            PaperFormat::A2 => (Unit::Millimeter.to_points(420.0, 72), Unit::Millimeter.to_points(594.0, 72)),
            PaperFormat::A3 => (Unit::Millimeter.to_points(297.0, 72), Unit::Millimeter.to_points(420.0, 72)),
            PaperFormat::A4 => (Unit::Millimeter.to_points(210.0, 72), Unit::Millimeter.to_points(297.0, 72)),
            PaperFormat::A5 => (Unit::Millimeter.to_points(148.0, 72), Unit::Millimeter.to_points(210.0, 72)),
            PaperFormat::A6 => (Unit::Millimeter.to_points(105.0, 72), Unit::Millimeter.to_points(148.0, 72)),
            
            // B-series
            PaperFormat::B0 => (Unit::Millimeter.to_points(1000.0, 72), Unit::Millimeter.to_points(1414.0, 72)),
            PaperFormat::B1 => (Unit::Millimeter.to_points(707.0, 72), Unit::Millimeter.to_points(1000.0, 72)),
            PaperFormat::B2 => (Unit::Millimeter.to_points(500.0, 72), Unit::Millimeter.to_points(707.0, 72)),
            PaperFormat::B3 => (Unit::Millimeter.to_points(353.0, 72), Unit::Millimeter.to_points(500.0, 72)),
            PaperFormat::B4 => (Unit::Millimeter.to_points(250.0, 72), Unit::Millimeter.to_points(353.0, 72)),
            PaperFormat::B5 => (Unit::Millimeter.to_points(176.0, 72), Unit::Millimeter.to_points(250.0, 72)),
            PaperFormat::B6 => (Unit::Millimeter.to_points(125.0, 72), Unit::Millimeter.to_points(176.0, 72)),
            
            // US Letter / Legal
            PaperFormat::Letter => (Unit::Inch.to_points(8.5, 72), Unit::Inch.to_points(11.0, 72)),
            PaperFormat::Legal => (Unit::Inch.to_points(8.5, 72), Unit::Inch.to_points(14.0, 72)),
            
            PaperFormat::Custom { width_pt, height_pt } => (width_pt, height_pt),
        }
    }
}
