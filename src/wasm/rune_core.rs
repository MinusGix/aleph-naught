use runestick::{ContextError, Module};

// Most of this is slightly modified from the rune wasm example

/// Overwrite existing standard functions to perform correctly in wasm.
pub fn create_module() -> Result<Module, ContextError> {
    let mut module = Module::new(&["std"]);
    module.function(&["print"], rune_print)?;
    module.function(&["println"], rune_println)?;
    module.raw_fn(&["dbg"], rune_dbg)?;
    Ok(module)
}

thread_local! {
    static OUT: std::cell::RefCell<std::io::Cursor<Vec<u8>>> = std::cell::RefCell::new(std::io::Cursor::new(Vec::new()));
}

/// Drain the output stored in the thread local `OUT`.
/// If non-utf8 output, then it will return None.
pub fn drain_output() -> Option<String> {
    OUT.with(|out| {
        let mut out = out.borrow_mut();
        let out = std::mem::take(&mut *out).into_inner();
        String::from_utf8(out).ok()
    })
}

fn rune_print(message: &str) -> Result<(), runestick::Panic> {
    use std::io::Write;

    OUT.with(|out| {
        let mut out = out.borrow_mut();
        write!(out, "{}", message).map_err(runestick::Panic::custom)
    })
}

fn rune_println(message: &str) -> Result<(), runestick::Panic> {
    use std::io::Write;

    OUT.with(|out| {
        let mut out = out.borrow_mut();
        writeln!(out, "{}", message).map_err(runestick::Panic::custom)
    })
}

fn rune_dbg(stack: &mut runestick::Stack, args: usize) -> Result<(), runestick::VmError> {
    use std::io::Write;

    OUT.with(|out| {
        let mut out = out.borrow_mut();

        for value in stack.drain_stack_top(args)? {
            writeln!(out, "{:?}", value).map_err(runestick::VmError::panic)?;
        }

        stack.push(runestick::Value::Unit);
        Ok(())
    })
}
