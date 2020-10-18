use runestick::{ContextError, Module, VmError};

pub fn set_panic_hook() {
    // When the `console_error_panic_hook` feature is enabled, we can call the
    // `set_panic_hook` function at least once during initialization, and then
    // we will get better error messages if our code ever panics.
    //
    // For more details see
    // https://github.com/rustwasm/console_error_panic_hook#readme
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();
}

pub mod string;

pub const MODULE_NAME: &str = "Util";

pub fn create_modules() -> Result<[Module; 2], ContextError> {
    let mut module = Module::new(&[MODULE_NAME]);

    let string_module = Module::new(&[MODULE_NAME, string::MODULE_NAME]);
    string::register(&mut module)?;

    module.inst_fn("map", map_vec)?;

    Ok([module, string_module])
}

/// Map a Vec<Value> -> Vec<Value>
/// At the current moment, rune only has a POC for iterator suppport
/// (https://github.com/rune-rs/rune/pull/156) and as normal rust iterators are, they are verbose
fn map_vec(vec: runestick::Vec, func: runestick::Function) -> Result<runestick::Vec, VmError> {
    let mut result = Vec::with_capacity(vec.len());
    for value in vec {
        result.push(func.call((value,))?);
    }
    Ok(runestick::Vec::from(result))
}
