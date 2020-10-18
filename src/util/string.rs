use runestick::{ContextError, Module};

pub const MODULE_NAME: &str = "String";

pub fn register(module: &mut Module) -> Result<(), ContextError> {
    module.inst_fn("capitalize", |mut text: String| -> String {
        capitalize(text.as_mut_str());
        text
    })?;
    Ok(())
}

/// Only capitalizes ascii.
/// ```rust
/// let text: &mut str = "hello";
/// capitalize(text);
/// assert_eq!(text, "Hello");
/// ```
pub fn capitalize(text: &mut str) {
    if !text.is_empty() {
        text[0..1].make_ascii_uppercase();
    }
}
