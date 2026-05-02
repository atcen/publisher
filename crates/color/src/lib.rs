use publisher_core::Color as CoreColor;
use lcms2::{Profile, Transform, Intent, PixelFormat};
use std::sync::Arc;

pub fn init() {
    publisher_core::init();
    println!("Publisher Color Initialized");
}

pub struct ColorEngine {
    rgb_profile: Arc<Profile>,
    cmyk_profile: Arc<Profile>,
    rgb_to_cmyk: Option<Transform<f64, f64, 3, 4>>,
    cmyk_to_rgb: Option<Transform<f64, f64, 4, 3>>,
}

impl ColorEngine {
    pub fn new() -> Result<Self, String> {
        let rgb_profile = Arc::new(Profile::new_srgb());
        let cmyk_profile = Arc::new(Profile::new_srgb()); // Placeholder
        
        let mut engine = Self {
            rgb_profile,
            cmyk_profile,
            rgb_to_cmyk: None,
            cmyk_to_rgb: None,
        };
        
        engine.rebuild_transforms()?;
        Ok(engine)
    }

    fn rebuild_transforms(&mut self) -> Result<(), String> {
        self.rgb_to_cmyk = Some(Transform::new(
            &self.rgb_profile,
            PixelFormat::RGB_DBL,
            &self.cmyk_profile,
            PixelFormat::CMYK_DBL,
            Intent::RelativeColorimetric,
        ).map_err(|e| format!("Failed to create RGB->CMYK transform: {:?}", e))?);

        self.cmyk_to_rgb = Some(Transform::new(
            &self.cmyk_profile,
            PixelFormat::CMYK_DBL,
            &self.rgb_profile,
            PixelFormat::RGB_DBL,
            Intent::RelativeColorimetric,
        ).map_err(|e| format!("Failed to create CMYK->RGB transform: {:?}", e))?);

        Ok(())
    }

    pub fn set_cmyk_profile(&mut self, data: &[u8]) -> Result<(), String> {
        let profile = Profile::new_icc(data)
            .map_err(|e| format!("Failed to load ICC profile: {:?}", e))?;
        self.cmyk_profile = Arc::new(profile);
        self.rebuild_transforms()
    }

    pub fn rgb_to_cmyk(&self, r: f64, g: f64, b: f64) -> Result<(f64, f64, f64, f64), String> {
        if let Some(t) = &self.rgb_to_cmyk {
            let input = [[r, g, b]];
            let mut output = [[0.0, 0.0, 0.0, 0.0]];
            t.transform(&input, &mut output);
            Ok((output[0][0], output[0][1], output[0][2], output[0][3]))
        } else {
            Err("Transform not initialized".to_string())
        }
    }

    pub fn cmyk_to_rgb(&self, c: f64, m: f64, y: f64, k: f64) -> Result<(f64, f64, f64), String> {
        if let Some(t) = &self.cmyk_to_rgb {
            let input = [[c, m, y, k]];
            let mut output = [[0.0, 0.0, 0.0]];
            t.transform(&input, &mut output);
            Ok((output[0][0], output[0][1], output[0][2]))
        } else {
            Err("Transform not initialized".to_string())
        }
    }

    pub fn convert_core_color(&self, color: &CoreColor) -> CoreColor {
        match color {
            CoreColor::Rgb { r, g, b } => {
                if let Ok((c, m, y, k)) = self.rgb_to_cmyk(*r, *g, *b) {
                    CoreColor::Cmyk { c, m, y, k }
                } else {
                    color.clone()
                }
            }
            CoreColor::Cmyk { c, m, y, k } => {
                if let Ok((r, g, b)) = self.cmyk_to_rgb(*c, *m, *y, *k) {
                    CoreColor::Rgb { r, g, b }
                } else {
                    color.clone()
                }
            }
            _ => color.clone(),
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_color_init() {
        init();
    }

    #[test]
    fn test_rgb_to_cmyk_placeholder() {
        let engine = ColorEngine::new().unwrap();
        // Note: With sRGB as both profiles, it won't be a real CMYK conversion, 
        // but it tests the LCMS2 transform logic.
        let res = engine.rgb_to_cmyk(1.0, 0.0, 0.0);
        assert!(res.is_ok());
    }
}
