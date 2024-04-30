pub struct ColorScheme {
    pub game_bg: termion::color::Rgb,
    pub cell_base: termion::color::Rgb,
    pub cell_row_active: termion::color::Rgb,
    pub cell_active: termion::color::Rgb,
    pub cell_in_word: termion::color::Rgb,
    pub cell_correct: termion::color::Rgb,
    pub text_base: termion::color::Rgb,
    pub text_inverted: termion::color::Rgb,
}

trait IntoTermionRgb {
    fn to_termion_rgb(&self) -> termion::color::Rgb;
}

// Default color scheme is built with catppuccin :)
impl IntoTermionRgb for catppuccin::Color {
    fn to_termion_rgb(&self) -> termion::color::Rgb {
        let catppuccin::Rgb { r, g, b } = self.rgb;
        termion::color::Rgb(r, g, b)
    }
}

impl From<catppuccin::Flavor> for ColorScheme {
    fn from(value: catppuccin::Flavor) -> Self {
        let c = value.colors;
        Self {
            game_bg: c.base.to_termion_rgb(),
            cell_base: c.surface1.to_termion_rgb(),
            cell_row_active: c.surface2.to_termion_rgb(),
            cell_active: c.overlay1.to_termion_rgb(),
            cell_in_word: c.yellow.to_termion_rgb(),
            cell_correct: c.green.to_termion_rgb(),
            text_base: c.text.to_termion_rgb(),
            text_inverted: c.base.to_termion_rgb(),
        }
    }
}
