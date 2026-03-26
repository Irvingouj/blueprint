use crate::error::Result;
use crate::skill_text::SKILL_TEXT;

pub fn run() -> Result<()> {
    print!("{SKILL_TEXT}");
    Ok(())
}
