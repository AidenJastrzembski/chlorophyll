use crate::models::comptime_template::ComptimeTemplate;
use anyhow::Result;

pub struct Waybar;

impl ComptimeTemplate for Waybar {
    const NAME: &'static str = "waybar";
    const TEMPLATE: &'static str = "waybar";
    const RELOAD: Option<&'static str> = None;

    fn install() -> Result<()> {
        Ok(())
    }
}
