use anyhow::Result;

// Ok i know the naming is a big zig-pilled but it kinda makes sense
pub trait ComptimeTemplate {
    // the name of the service the template is for
    const NAME: &'static str;
    // the name of the file which will be put in the templates dir
    const TEMPLATE: &'static str;
    // the reload command which will be added to the config
    const RELOAD: Option<&'static str>;

    // fn that will create the template in the templates dir
    fn install() -> Result<()> {
        Ok(())
    }
}
