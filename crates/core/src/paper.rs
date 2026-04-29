use crate::units::{Unit, Pt};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PaperFormat {
    A0, A1, A2, A3, A4, A5, A6,
    B0, B1, B2, B3, B4, B5, B6,
    Letter,
    Legal,
    Custom { width_pt: Pt, height_pt: Pt },
}

impl PaperFormat {
    pub fn dimensions_pt(self) -> (Pt, Pt) {
        match self {
            // A-series
            PaperFormat::A0 => (Unit::Millimeter.to_points(841.0, None), Unit::Millimeter.to_points(1189.0, None)),
            PaperFormat::A1 => (Unit::Millimeter.to_points(594.0, None), Unit::Millimeter.to_points(841.0, None)),
            PaperFormat::A2 => (Unit::Millimeter.to_points(420.0, None), Unit::Millimeter.to_points(594.0, None)),
            PaperFormat::A3 => (Unit::Millimeter.to_points(297.0, None), Unit::Millimeter.to_points(420.0, None)),
            PaperFormat::A4 => (Unit::Millimeter.to_points(210.0, None), Unit::Millimeter.to_points(297.0, None)),
            PaperFormat::A5 => (Unit::Millimeter.to_points(148.0, None), Unit::Millimeter.to_points(210.0, None)),
            PaperFormat::A6 => (Unit::Millimeter.to_points(105.0, None), Unit::Millimeter.to_points(148.0, None)),

            // B-series
            PaperFormat::B0 => (Unit::Millimeter.to_points(1000.0, None), Unit::Millimeter.to_points(1414.0, None)),
            PaperFormat::B1 => (Unit::Millimeter.to_points(707.0, None), Unit::Millimeter.to_points(1000.0, None)),
            PaperFormat::B2 => (Unit::Millimeter.to_points(500.0, None), Unit::Millimeter.to_points(707.0, None)),
            PaperFormat::B3 => (Unit::Millimeter.to_points(353.0, None), Unit::Millimeter.to_points(500.0, None)),
            PaperFormat::B4 => (Unit::Millimeter.to_points(250.0, None), Unit::Millimeter.to_points(353.0, None)),
            PaperFormat::B5 => (Unit::Millimeter.to_points(176.0, None), Unit::Millimeter.to_points(250.0, None)),
            PaperFormat::B6 => (Unit::Millimeter.to_points(125.0, None), Unit::Millimeter.to_points(176.0, None)),

            // US Letter / Legal
            PaperFormat::Letter => (Unit::Inch.to_points(8.5, None), Unit::Inch.to_points(11.0, None)),
            PaperFormat::Legal => (Unit::Inch.to_points(8.5, None), Unit::Inch.to_points(14.0, None)),

            PaperFormat::Custom { width_pt, height_pt } => (width_pt, height_pt),
        }
    }
}
