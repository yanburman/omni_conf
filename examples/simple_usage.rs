use omni_conf::{ConfigManager, Format};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Default)]
struct UserSettings {
    dark_mode: bool,
}

fn main() -> omni_conf::Result<()> {
    let config = ConfigManager::builder("com", "AcmeCorp", "SuperApp")
        .with_format(Format::Toml)
        .build()?;

    let mut settings: UserSettings = config.load_or_default()?;
    settings.dark_mode = true;
    config.save(&settings)?;

    Ok(())
}
