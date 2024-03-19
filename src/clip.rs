use std::error::Error;

use cli_clipboard::{ClipboardContext, ClipboardProvider};

pub fn copy_to_clipboard(link: String) -> Result<(), Box<dyn Error>> {
    let mut ctx: ClipboardContext = match ClipboardProvider::new() {
        Ok(ctx) => ctx,
        Err(e) => {
            return Err(format!("Failed to copy to clipboard:\n{}", e).into());
        }
    };
    if let Err(e) = ctx.set_contents(link.clone()) {
        return Err(format!("Failed to copy to clipboard:\n{}", e).into());
    }
    Ok(())
}
