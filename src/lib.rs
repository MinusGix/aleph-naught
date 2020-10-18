mod engine;
mod util;
mod wasm;

use std::sync::Arc;

use rune::EmitDiagnostics;

use wasm_bindgen::prelude::*;

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

fn create_rune(sources: &mut rune::Sources) -> Result<runestick::Vm, String> {
    let mut warnings = rune::Warnings::new();
    let mut errors = rune::Errors::new();

    let context = match create_rune_context() {
        Ok(context) => context,
        Err(err) => return Err(format!("{}", err)),
    };

    let unit = rune::load_sources(
        &context,
        &rune::Options::default(),
        sources,
        &mut errors,
        &mut warnings,
    );

    let mut writer = rune::termcolor::Buffer::no_color();

    if !errors.is_empty() {
        errors
            .emit_diagnostics(&mut writer, &sources)
            .expect("emitting to buffer should not fail");
        let mut string =
            String::from_utf8(writer.into_inner()).expect("Expected diagnostics to be valid utf8");
        let new_len = string.trim_end().len();
        string.truncate(new_len);
        return Err(string);
    // let mut error_string = "Rune Errors: ".to_owned();
    // for error in errors {
    //     error_string.push_str(format!("{}\n", error.to_string()).as_str());
    // }
    // return Err(error_string);
    } else if !warnings.is_empty() {
        warnings
            .emit_diagnostics(&mut writer, &sources)
            .expect("emitting to buffer should not fail");
        let mut string =
            String::from_utf8(writer.into_inner()).expect("Expected diagnostics to be valid utf8");
        let new_len = string.trim_end().len();
        string.truncate(new_len);
        return Err(string);
        // let mut warning_string = "Rune Warnings: ".to_owned();
        // for warning in &warnings {
        //     warning_string.push_str(format!("{}\n", warning).as_str());
        // }
        // return Err(warning_string);
    }

    let unit = unit.unwrap();

    let vm = runestick::Vm::new(Arc::new(context), Arc::new(unit));

    Ok(vm)
}
fn create_rune_context() -> Result<runestick::Context, runestick::ContextError> {
    let mut context = runestick::Context::with_config(false)?;
    context.install(&wasm::rune_core::create_module()?)?;
    context.install(&wasm::rune_lib::create_module()?)?;
    context.install(&engine::create_module()?)?;
    for module in util::create_modules()?.iter() {
        context.install(&module)?;
    }
    Ok(context)
}

#[wasm_bindgen]
pub async fn start(info: JsValue) -> Result<JsValue, JsValue> {
    // Provide better panic information.
    util::set_panic_hook();

    // Alert that we are starting. This should be done before essentially all setup so as to
    // allow an idea of the progress.
    log_info("Starting");

    // The info that we receive should be an object, as that allows it to pass the information
    // TODO: Potentially make a class? There isn't a great need for it though.
    if !info.is_object() {
        return Err(JsValue::from_str("Failure, info was not an object."));
    }

    let info = engine::UserInfo::from_js_object(info)
        .await
        .expect("Info passed into start was not of correct format");

    let mut state = engine::State::new(info);

    let vm = create_rune(&mut state.uinfo.sources.sources)?;
    log_info("Created virtual machine");

    let _result = match vm.call(&["entry"], (state,)) {
        Ok(value) => value,
        Err(err) => return Err(format!("[VMError::entry]: {}", err).into()),
    };

    if let Some(output) = wasm::rune_core::drain_output() {
        log(output.as_str());
    }

    Ok(JsValue::null())
}

fn clear_element(element: web_sys::Element) {
    element.set_inner_html("");
}

pub fn log_info(text: &str) {
    log2("[Aleph-Naught]", text);
}

pub fn log(text: &str) {
    web_sys::console::log_1(&JsValue::from_str(text));
}
pub fn log2(prefix: &str, text: &str) {
    web_sys::console::log_2(&JsValue::from_str(prefix), &JsValue::from_str(text));
}

pub fn handle_vm_result<V>(v: Result<V, runestick::VmError>) -> V {
    match v {
        Ok(v) => v,
        Err(err) => panic!("[VMError]: {}", err),
    }
}
