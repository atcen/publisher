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
}

impl ColorEngine {
    pub fn new() -> Result<Self, String> {
        // Load default profiles (sRGB and a generic CMYK)
        let rgb_profile = Profile::new_srgb();
        // For CMYK, we would ideally load a real profile like FOGRA39.
        // For this implementation, we'll use a built-in or dummy if available, 
        // or just expect the user to provide one.
        // LCMS2 doesn't have a built-in "new_cmyk", so we'll try to load a placeholder
        // or just initialize with sRGB for now and allow overrides.
        
        Ok(Self {
            rgb_profile: Arc::new(rgb_profile),
            cmyk_profile: Arc::new(Profile::new_srgb()), // Placeholder
        })
    }

    pub fn set_cmyk_profile(&mut self, data: &[u8]) -> Result<(), String> {
        let profile = Profile::new_icc(data)
            .map_err(|e| format!("Failed to load ICC profile: {:?}", e))?;
        self.cmyk_profile = Arc::new(profile);
        Ok(())
    }

    pub fn rgb_to_cmyk(&self, r: f64, g: f64, b: f64) -> Result<(f64, f64, f64, f64), String> {
        let t = Transform::new(
            &self.rgb_profile,
            PixelFormat::RGB_DBL,
            &self.cmyk_profile,
            PixelFormat::CMYK_DBL,
            Intent::RelativeColorimetric,
        ).map_err(|e| format!("Failed to create transform: {:?}", e))?;

        let input = [r, g, b];
        let mut output = [0.0, 0.0, 0.0, 0.0];
        t.transform(&[input], &mut [output]);

        Ok((output[0], output[1], output[2], output[3]))
    }

    pub fn cmyk_to_rgb(&self, c: f64, m: f64, y: f64, k: f64) -> Result<(f64, f64, f64), String> {
        let t = Transform::new(
            &self.cmyk_profile,
            PixelFormat::CMYK_DBL,
            &self.rgb_profile,
            PixelFormat::RGB_DBL,
            Intent::RelativeColorimetric,
        ).map_err(|e| format!("Failed to create transform: {:?}", e))?;

        let input = [c, m, y, k];
        let mut output = [0.0, 0.0, 0.0];
        t.transform(&[input], &mut [output]);

        Ok((output[0], output[1], output[2]))
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
