use crate::templates::waybar::Waybar;

pub mod renderer;
pub mod rofi;
pub mod waybar;

static WAYBAR: waybar::Waybar = Waybar;
static ROFI: rofi::Rofi = rofi::Rofi;
