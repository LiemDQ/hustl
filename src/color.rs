pub type Color = [f32; 4]; //r g b w

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct ColorTheme {
    background_1: Color,
    background_2: Color,
    background_3: Color,
    background_4: Color,
    model_key: Color,
    model_fill: Color,
    model_base: Color,
}

impl ColorTheme {
    pub fn get_background_colors(&self) -> [Color;4] {
        [self.background_1, self.background_2, self.background_3, self.background_4].map(ColorTheme::rgb_to_srgb)
    }

    pub fn get_model_colors(&self) -> [Color;3] {
        [self.model_key, self.model_fill, self.model_base].map(ColorTheme::rgb_to_srgb)
    }

    /// Approximation of the mapping from RGB to sRGB, which is used by most monitors
    fn rgb_to_srgb(color: Color) -> Color {
        color.map(|val| val.powf(2.2))
    }
}

pub enum Theme {
    Light,
    Dark,
    Solarized,
}

impl Theme {
    pub fn get_values(&self) -> ColorTheme {
        match self {
            Theme::Dark => {
                ColorTheme {
                    background_1: [0.05, 0.06, 0.10, 1.0],
                    background_2: [0.05, 0.06, 0.10, 1.0],
                    background_3: [0.17, 0.22, 0.29, 1.0],
                    background_4: [0.17, 0.22, 0.29, 1.0],
                    model_key: [0.99, 0.96, 0.89, 1.0],
                    model_fill: [0.93, 0.91, 0.84, 1.0],
                    model_base: [0.0, 0.0, 0.0, 1.0],
                }
            },
            Theme::Light => {
                ColorTheme {
                    background_1: [0.63, 0.83, 1.0, 1.0],
                    background_2: [0.63, 0.83, 1.0, 1.0],
                    background_3: [0.94, 0.95, 0.96, 1.0],
                    background_4: [0.94, 0.95, 0.96, 1.0],
                    model_key: [0.41, 0.47, 0.52, 1.0],
                    model_fill: [0.6, 0.65, 0.69, 1.0],
                    model_base: [0.0, 0.0, 0.0, 1.0],
                }
            },
            Theme::Solarized => {
                ColorTheme {
                    background_1: [0.0, 0.08, 0.10, 1.0],
                    background_2: [0.0, 0.08, 0.10, 1.0],
                    background_3: [0.0, 0.20, 0.25, 1.0],
                    background_4: [0.0, 0.20, 0.25, 1.0],
                    model_key: [0.99, 0.96, 0.89, 1.0],
                    model_fill: [0.93, 0.91, 0.84, 1.0],
                    model_base: [0.41, 0.48, 0.51, 1.0],
                }
            }
        }
    }
}