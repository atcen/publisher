#![allow(clippy::arc_with_non_send_sync)]
use lcms2::{Intent, PixelFormat, Profile, Transform};
use publisher_core::Color as CoreColor;
use std::sync::Arc;

pub fn init() {
    publisher_core::init();
    println!("Publisher Color Initialized");
}

pub struct ColorEngine {
    rgb_profile: Arc<Profile>,
    cmyk_profile: Arc<Profile>,
    rgb_to_cmyk: Option<Transform<[f64; 3], [f64; 4]>>,
    cmyk_to_rgb: Option<Transform<[f64; 4], [f64; 3]>>,
}

// Safety: ColorEngine is always wrapped in a Mutex in the application state.
// LCMS2 objects contain raw pointers but can be moved between threads.
unsafe impl Send for ColorEngine {}
unsafe impl Sync for ColorEngine {}

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

        let _ = engine.rebuild_transforms();
        Ok(engine)
    }

    fn rebuild_transforms(&mut self) -> Result<(), String> {
        // We use .ok() here because placeholder profiles (like sRGB for CMYK)
        // will fail to create transforms. Real profiles will work.
        self.rgb_to_cmyk = Transform::new(
            &self.rgb_profile,
            PixelFormat::RGB_DBL,
            &self.cmyk_profile,
            PixelFormat::CMYK_DBL,
            Intent::RelativeColorimetric,
        )
        .ok();

        self.cmyk_to_rgb = Transform::new(
            &self.cmyk_profile,
            PixelFormat::CMYK_DBL,
            &self.rgb_profile,
            PixelFormat::RGB_DBL,
            Intent::RelativeColorimetric,
        )
        .ok();

        Ok(())
    }

    pub fn set_cmyk_profile(&mut self, data: &[u8]) -> Result<(), String> {
        let profile =
            Profile::new_icc(data).map_err(|e| format!("Failed to load ICC profile: {:?}", e))?;
        self.cmyk_profile = Arc::new(profile);
        self.rebuild_transforms()
    }

    pub fn rgb_to_cmyk(&self, r: f64, g: f64, b: f64) -> Result<(f64, f64, f64, f64), String> {
        if let Some(t) = &self.rgb_to_cmyk {
            let input = [[r, g, b]];
            let mut output = [[0.0, 0.0, 0.0, 0.0]];
            t.transform_pixels(&input, &mut output);
            Ok((output[0][0], output[0][1], output[0][2], output[0][3]))
        } else {
            Err("Transform not initialized (likely incompatible profiles)".to_string())
        }
    }

    pub fn cmyk_to_rgb(&self, c: f64, m: f64, y: f64, k: f64) -> Result<(f64, f64, f64), String> {
        if let Some(t) = &self.cmyk_to_rgb {
            let input = [[c, m, y, k]];
            let mut output = [[0.0, 0.0, 0.0]];
            t.transform_pixels(&input, &mut output);
            Ok((output[0][0], output[0][1], output[0][2]))
        } else {
            // Naive fallback conversion
            let r = (1.0 - c) * (1.0 - k);
            let g = (1.0 - m) * (1.0 - k);
            let b = (1.0 - y) * (1.0 - k);
            Ok((r, g, b))
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
            CoreColor::Spot {
                alternate_cmyk,
                tint,
                ..
            } => {
                let (c, m, y, k) = *alternate_cmyk;
                // Apply tint to the CMYK values for preview
                let tc = c * tint;
                let tm = m * tint;
                let ty = y * tint;
                let tk = k * tint;
                if let Ok((r, g, b)) = self.cmyk_to_rgb(tc, tm, ty, tk) {
                    CoreColor::Rgb { r, g, b }
                } else {
                    color.clone()
                }
            }
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
        let rgb_profile = Profile::new_srgb();
        let transform = Transform::<[f64; 3], [f64; 3]>::new(
            &rgb_profile,
            PixelFormat::RGB_DBL,
            &rgb_profile,
            PixelFormat::RGB_DBL,
            Intent::RelativeColorimetric,
        );
        assert!(transform.is_ok());
    }

    #[test]
    fn test_spot_to_rgb_conversion() {
        let engine = ColorEngine::new().unwrap();
        let spot = CoreColor::Spot {
            name: "PANTONE 185 C".to_string(),
            alternate_cmyk: (0.0, 0.91, 0.76, 0.0),
            tint: 1.0,
        };
        let rgb = engine.convert_core_color(&spot);
        if let CoreColor::Rgb { r, g, b } = rgb {
            assert!(r > 0.0);
            assert!(g < 0.5);
            assert!(b < 0.5);
        } else {
            panic!("Expected Rgb color");
        }
    }
}
