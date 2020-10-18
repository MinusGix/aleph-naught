use wasm_bindgen::prelude::*;

use crate::{log, log2};

#[wasm_bindgen]
extern "C" {
    pub fn alert(s: &str);
}

pub fn create_module() -> Result<runestick::Module, runestick::ContextError> {
    let mut module = runestick::Module::new(&["wasm"]);
    module.function(&["alert"], alert)?;
    module.function(&["log1"], log)?;
    module.function(&["log_info1"], |text: &str| {
        log2("[Aleph:Script]", text);
    })?;
    Ok(module)
}
