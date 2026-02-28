use crate::models::comptime_template::ComptimeTemplate;
use anyhow::Result;

pub struct Rofi;

impl ComptimeTemplate for Rofi {
    const NAME: &'static str = "rofi";
    const TEMPLATE: &'static str = "colors.rasi";
    const RELOAD: Option<&'static str> = None;

    fn install() -> Result<()> {
        Ok(())
    }
}
