use std::{cell::RefCell, collections::HashMap, sync::Arc};

use runestick::{Any, Shared};
use wasm::util::document;
use wasm_bindgen::{prelude::Closure, JsCast, JsValue};

use crate::{
    clear_element, handle_vm_result,
    wasm::{self, rune_lib::alert},
};

pub const MODULE_NAME: &str = "Engine";
pub fn create_module() -> Result<runestick::Module, runestick::ContextError> {
    let mut module = runestick::Module::new(&[MODULE_NAME]);
    register(&mut module)?;
    Ok(module)
}

pub fn register(module: &mut runestick::Module) -> Result<(), runestick::ContextError> {
    Button::register(module)?;
    State::register(module)?;
    Ok(())
}

#[derive(Debug, Any)]
pub struct Scenes {
    scenes: HashMap<String, Scene>,
}
impl Default for Scenes {
    fn default() -> Self {
        Self {
            scenes: HashMap::with_capacity(64),
        }
    }
}
#[derive(Debug, Any)]
pub struct Scene {
    display_callback: runestick::Function,
}
impl Scene {
    pub fn new(display_callback: runestick::Function) -> Self {
        Self { display_callback }
    }
}

#[derive(Debug, Any)]
pub struct Button {
    /// The text displayed for the button
    pub text: String,
    /// Data that should be used when it is activated
    pub on_activate_data: runestick::Value,
}
impl Button {
    pub fn new(text: String, on_activate_data: runestick::Value) -> Self {
        Self {
            text,
            on_activate_data,
        }
    }

    pub fn register(module: &mut runestick::Module) -> Result<(), runestick::ContextError> {
        module.ty::<Self>()?;
        module.function(&["Button", "new"], Self::new)?;
        Ok(())
    }
}

#[derive(Debug)]
pub struct TextElement {
    element: web_sys::HtmlElement,
}
impl TextElement {
    pub fn new(element: web_sys::HtmlElement) -> Self {
        Self { element }
    }

    pub fn set_text(&self, text: &str) {
        self.element.set_inner_text(text);
    }

    pub fn clear_text(&self) {
        self.element.set_inner_text("");
    }

    pub fn append_text(&self, text: &str) {
        let text = format!("{}{}", self.element.inner_text(), text);
        self.set_text(&text);
    }
}

/// Information passed in by the user when creating the instance.
/// Its definition obviously depends on where this is being hosted.
#[derive(Debug, Any)]
pub struct UserInfo {
    /// Title text
    pub title_element: TextElement,
    /// Display text
    pub text_element: TextElement,
    /// Inputs.
    /// Text inputs, buttons, etc.
    pub input_element: web_sys::Element,
    /// The sources and information about them
    pub sources: SourceUserInfo,
}
impl UserInfo {
    const TITLE_ELEMENT_KEY: &'static str = "title_element";
    const TEXT_ELEMENT_KEY: &'static str = "text_element";
    const INPUT_ELEMENT_KEY: &'static str = "input_element";

    // TODO: better error type than a `JsValue`
    pub async fn from_js_object(info: JsValue) -> Result<Self, JsValue> {
        if !info.is_object() {
            return Err(JsValue::from_str(
                "Failure to create startup information. Info was not an object",
            ));
        }

        let title_element = wasm::util::get_object_property::<web_sys::HtmlElement>(
            &info,
            Self::TITLE_ELEMENT_KEY,
        )?;
        let title_element = TextElement::new(title_element);

        let text_element =
            wasm::util::get_object_property::<web_sys::HtmlElement>(&info, Self::TEXT_ELEMENT_KEY)?;
        let text_element = TextElement::new(text_element);

        let input_element =
            wasm::util::get_object_property::<web_sys::Element>(&info, Self::INPUT_ELEMENT_KEY)?;

        let source_user_info = SourceUserInfo::try_from_js_value(&info).await?;

        Ok(Self {
            title_element,
            text_element,
            input_element,
            sources: source_user_info,
        })
    }
}

// TODO: associate more debug info with each field of this if it is targeting wasm
pub enum SourceInfoError {
    /// An error in getting a value that should be an iterator
    IterFailure {
        key: &'static str,
    },
    /// Experienced an error while iterating from the iterator
    ActiveIterFailure {
        key: &'static str,
    },
    ExpectedString,
    RequestCreationFailure,
    RequestFailure,
    BadRequestBody,
}
impl Into<String> for SourceInfoError {
    fn into(self) -> String {
        match self {
            Self::IterFailure { key } => format!("Expected property '{}' to be iterable", key),
            Self::ActiveIterFailure { key } => format!(
                "While iterating over '{}', experienced error in getting value",
                key
            ),
            Self::ExpectedString => {
                "Expected string, but got non-string value. (Good luck).".to_owned()
            }
            Self::RequestCreationFailure => {
                "Failed to create network request before even sending it.".to_owned()
            }
            Self::RequestFailure => "Failure in network request.".to_owned(),
            Self::BadRequestBody => "Failed to get request body".to_owned(),
        }
    }
}
impl From<SourceInfoError> for JsValue {
    fn from(error: SourceInfoError) -> JsValue {
        let error_message: String = error.into();
        JsValue::from_str(error_message.as_str())
    }
}

#[derive(Debug, Any)]
pub struct SourceUserInfo {
    pub sources: rune::Sources,
}
impl SourceUserInfo {
    // Files: `Iterator<Item=String>`
    const FILES_KEY: &'static str = "files";

    async fn try_from_js_value(value: &JsValue) -> Result<Self, SourceInfoError> {
        use crate::wasm::util::window;
        use js_sys::Reflect;
        use wasm_bindgen_futures::JsFuture;
        use web_sys::{Request, RequestInit, RequestMode, Response};

        let mut sources = rune::Sources::new();

        if let Ok(files) = Reflect::get(value, &JsValue::from_str(Self::FILES_KEY)) {
            // Extract the value from the conversion into an iterator
            // this allows us to not bother checking if it is an array of strings.
            // all it has to be is iteratable and each iteration item is a string
            let files = js_sys::try_iter(&files)
                .map_err(|_| SourceInfoError::IterFailure {
                    key: Self::FILES_KEY,
                })?
                .ok_or_else(|| SourceInfoError::IterFailure {
                    key: Self::FILES_KEY,
                })?;

            // Create the request options outside of the loop as it is the same very time.
            let mut request_options = RequestInit::new();
            request_options.method("GET");
            request_options.mode(RequestMode::Cors);
            // TODO: Collect all of these into a collection and use promise.all on them so that
            // they may complete as they wish
            for file in files {
                // Extract the value from the iterator
                let file: JsValue = file.map_err(|_| SourceInfoError::ActiveIterFailure {
                    key: Self::FILES_KEY,
                })?;
                // Get the value as a string. This is the place we should fetch the code from.
                let file: String = file.as_string().ok_or(SourceInfoError::ExpectedString)?;

                // Create the network Request object
                let request: Request = Request::new_with_str_and_init(&file, &request_options)
                    .map_err(|_| SourceInfoError::RequestCreationFailure)?;
                // Fetch using the request, and turn it into a JsFuture so that it can be awaited
                let response = JsFuture::from(window().fetch_with_request(&request))
                    .await
                    .map_err(|_| SourceInfoError::RequestFailure)?;
                // Convert the response JsValue into a Response object.
                let response: Response = response
                    .dyn_into()
                    .map_err(|_| SourceInfoError::RequestFailure)?;

                let code: JsValue = JsFuture::from(
                    response
                        .text()
                        .map_err(|_| SourceInfoError::BadRequestBody)?,
                )
                .await
                .map_err(|_| SourceInfoError::BadRequestBody)?;
                let code: String = code.as_string().ok_or(SourceInfoError::BadRequestBody)?;

                sources.insert(runestick::Source::new(file, code));
            }
        }

        Ok(SourceUserInfo { sources })
    }
}

#[derive(Debug, Any)]
pub struct State {
    pub current_scene: Option<String>,
    pub scenes: Shared<Scenes>,
    pub info: Shared<runestick::Object>,
    pub uinfo: UserInfo,
}
impl State {
    pub fn new(uinfo: UserInfo) -> Self {
        State {
            current_scene: None,
            scenes: Shared::new(Scenes::default()),
            info: Shared::new(runestick::Object::with_capacity(64)),
            uinfo,
        }
    }

    pub fn info(&self) -> Shared<runestick::Object> {
        self.info.clone()
    }

    // TODO: mess with this so that you can return the old object
    /// Overwrite the info
    pub fn overwrite_info(&mut self, object: runestick::Object) {
        *self
            .info
            .borrow_mut()
            .expect("Expected info to be available") = object;
    }

    pub fn goto(self, scene_name: String) {
        // Clone the scene so that we have another shared ref to it
        let scenes = self.scenes.clone();
        // Acquire a reference to the held scenes so that we can access the appropriate scene
        let scenes = scenes
            .borrow_ref()
            .expect("Expected to be able to borrow state's scenes so as to go towards a scene");
        // Get the scene which we desire to access
        let scene = scenes
            .scenes
            .get(&scene_name)
            .unwrap_or_else(|| panic!("Failed to find scene: '{}'", scene_name));
        let _: () = scene
            .display_callback
            .call((self,))
            .unwrap_or_else(|err| panic!("[Error in Scene: '{}']: {}", scene_name, err));
    }

    pub fn register(module: &mut runestick::Module) -> Result<(), runestick::ContextError> {
        module.ty::<Self>()?;
        module.inst_fn("ask_choice", Self::ask_choice)?;
        module.inst_fn("ask_input", Self::ask_input)?;
        // TODO: For some reason I can't register a getter that returns an `&mut Scenes`
        module.getter("info", Self::info)?;
        module.inst_fn(
            "add_scenes",
            |state: &mut State, new_scenes: Vec<(String, runestick::Function)>| {
                let mut scenes = state
                    .scenes
                    .borrow_mut()
                    .expect("Expected scenes to be available for modification");
                for (scene_name, display_callback) in new_scenes.into_iter() {
                    scenes
                        .scenes
                        .insert(scene_name, Scene::new(display_callback));
                }
            },
        )?;
        module.inst_fn("goto", Self::goto)?;
        module.inst_fn("overwrite_info", Self::overwrite_info)?;
        // TODO: for wasm, at least, we could easily clone the TextElements and just return that
        // through a getter. So: `state.text.set("blah");`, `state.text.append("bleh");`.
        // but, this may not work on other backends so it should wait.
        module.inst_fn("set_text", Self::set_text)?;
        module.inst_fn("append_text", Self::append_text)?;
        module.inst_fn("clear_text", Self::clear_text)?;
        module.inst_fn("set_title", Self::set_title)?;
        module.inst_fn("append_title", Self::append_title)?;
        module.inst_fn("clear_title", Self::clear_title)?;

        Ok(())
    }

    // TODO: maybe track that we are already asking a choice, so that we can't display > 1?
    // Just set the tracker to false upon clearing the buttons, after the first button is pressed
    pub fn ask_choice(&mut self, buttons: Vec<Button>, callback: runestick::Function) {
        // Store it atomically on the heap, so that we can give references to it to each button cb
        let buttons = Arc::new(RefCell::new(buttons));
        // Store atomically on heap, because each callback needs to be able to call it.
        let callback = Arc::new(callback);
        // TODO: perhaps tear apart the buttons vector, and just give ownership of the data over to
        // button closure?
        // Iterate over the buttons. We use the index as a way of referring to the specific button
        // that was pressed so that we do not need to store each button in an arc.
        for (index, button) in buttons.borrow().iter().enumerate() {
            let display_button = {
                let input_element = self.uinfo.input_element.clone();
                let text = button.text.as_str();
                let buttons = buttons.clone();
                let callback = callback.clone();
                DisplayButton::new(
                    text,
                    Closure::once(move |_event| {
                        // Extract the button, throwing away all the other buttons.
                        // SANENESS: Since only one of these callbacks should be called
                        // we take the value of buttons and extract the function as a value to avoid
                        // the pain of passing a reference or cloning (which we cannot do anyway)
                        let mut buttons = buttons.borrow_mut();
                        // TODO: we don't need to keep buttons around so this is a waste of energy
                        let button = buttons.swap_remove(index);
                        buttons.clear();
                        let on_activate_data: runestick::Value = button.on_activate_data;

                        // Specifically remove elements before calling anything.
                        // _Do not_ do this after calling, as that breaks things.
                        clear_element(input_element);
                        let result: Result<(), runestick::VmError> =
                            callback.call((on_activate_data,));
                        if let Err(err) = result {
                            panic!("[VMError:State::ask_choice]: {}", err);
                        }
                    }),
                )
            }
            .expect("Failed to create display button");
            display_button
                .add_to(&self.uinfo.input_element)
                .expect("Error in adding display button to inputs");
        }
    }

    /// Takes the default text, a function to check if the input is valid
    pub fn ask_input(
        &mut self,
        default_text: String,
        validator: runestick::Function,
        callback: runestick::Function,
    ) {
        let input_element = self.uinfo.input_element.clone();
        let line_input_element = LineInput::new(
            default_text.as_str(),
            move |text: String| -> bool { handle_vm_result(validator.call((text,))) },
            move |text: String| {
                // Remove the input element, because we only allow submitting it once.
                clear_element(input_element.clone());
                // Call the callback, getting the value but we don't really care about that.
                let result: Result<runestick::Value, runestick::VmError> = callback.call((text,));
                if let Err(err) = result {
                    panic!("[VMError:State::ask_input]: {}", err);
                }
            },
        )
        .expect("Failed to create line input");

        line_input_element
            .add_to(&self.uinfo.input_element)
            .expect("Error in adding line input to inputs");
    }

    pub fn set_text(&self, text: &str) {
        self.uinfo.text_element.set_text(text);
    }
    pub fn append_text(&self, text: &str) {
        self.uinfo.text_element.append_text(text);
    }
    pub fn clear_text(&self) {
        self.uinfo.text_element.clear_text();
    }

    pub fn set_title(&self, text: &str) {
        self.uinfo.title_element.set_text(text);
    }
    pub fn append_title(&self, text: &str) {
        self.uinfo.title_element.append_text(text);
    }
    pub fn clear_title(&self) {
        self.uinfo.title_element.clear_text();
    }
}

// Closure notes related to potential leaking memory:
// So, it appears that Rust-Wasm closures are kindof irritating (beyond their verbose syntax)
// When you create a closure:
// ```rust
// let closure = Closure::new(Box::new(|a: i32| -> i32 { a+1 }) as Box<dyn Fn(i32) -> i32>);
// ```
// Then you want to put it on something, like an event listener:
// `element.add_event_listener_with_callback("click", closure.as_ref().unchecked_ref())?;`
// Now, the documentation uses `closure.forget();`, I'd even say encourages it.
// This just leaks the memory for the closure, letting it live forever... which is bad.
// So, we want to keep the closure alive for as long as its needed by the js engine.
// My first idea was to wrap it in an (A)Rc, but that wouldn't work as you no longer have
// an alive strong Arc reference to the closure once it is converted to the &Function.
// Next, after delving through some issues in the github, I saw the recommended method as
// storing them somewhere so they aren't dropped. Okay, that's sensible.. but where do I
// store them? While it might be possible to set them up as individually allocated bits
// in a vector on my main state object, that is a *pain* and complicates code excessively.
// My closures also kindof need to destroy the element holding them, so they have to mess
// with the state when their called. I think this might work fine, but it sounds like it could
// easly lead to long debugging sessions as it tries to destroy the struct containing itself while
// in itself.
// So, essentially have to wait for weakrefs to be supported in all browsers to properly get basic
// garbage collection. So, until then it will likely leak memory.
// An FnOnce - but I believe only if called! - will be collected properly though.

pub struct DisplayButton {
    pub button: web_sys::HtmlButtonElement,
}
impl DisplayButton {
    /// Construct a new display button.
    /// The text
    pub fn new(
        text: &str,
        callback: Closure<dyn FnMut(web_sys::MouseEvent)>,
    ) -> Result<Self, JsValue> {
        let button = document()
            .create_element("button")?
            .dyn_into::<web_sys::HtmlButtonElement>()?;
        button.set_inner_text(text);

        // listen to click events
        button.add_event_listener_with_callback("click", callback.as_ref().unchecked_ref())?;
        callback.forget();

        Ok(Self { button })
    }

    pub fn add_to(&self, element: &web_sys::Element) -> Result<(), JsValue> {
        // TODO: better methd for adding it than this.
        let nodes = js_sys::Array::new();
        nodes.push(&JsValue::from(self.button.clone()));
        element.append_with_node(&nodes)
    }
}

const ENTER_KEYCODE: u32 = 13;
// TODO: include submit button?
pub struct LineInput {
    pub input: web_sys::HtmlInputElement,
}
impl LineInput {
    pub fn new<G, F>(text: &str, validator_callback: G, enter_callback: F) -> Result<Self, JsValue>
    where
        G: 'static + Fn(String) -> bool,
        F: 'static + Fn(String),
    {
        let input = document()
            .create_element("input")?
            .dyn_into::<web_sys::HtmlInputElement>()?;

        input.set_default_value(text);

        let input_a = input.clone();
        let closure = Closure::wrap(Box::new(move |event: web_sys::KeyboardEvent| {
            // Enter, but not shift-enter.
            if event.key_code() == ENTER_KEYCODE && !event.shift_key() {
                let input_value = input_a.value();
                // TODO: it would be nice to not have to duplicate the value
                if validator_callback(input_value.clone()) {
                    enter_callback(input_value);
                } else {
                    // TODO: don't just send out an alert that it is invalid..
                    // we could also do this whilst they type.
                    alert("Invalid input!");
                }
            }
        }) as Box<dyn Fn(_)>);

        // May leak memory. See Comment above this structure about a specific rust wasm problem
        let closure = closure.into_js_value();

        // SAFETY/SOUNDNESS: We *know* that the closure is a valid function, and so we hand it over.
        let function_ref: &js_sys::Function = closure.unchecked_ref();
        input.add_event_listener_with_callback("keyup", function_ref)?;

        Ok(Self { input })
    }

    pub fn add_to(&self, element: &web_sys::Element) -> Result<(), JsValue> {
        let nodes = js_sys::Array::new();
        nodes.push(&JsValue::from(self.input.clone()));
        element.append_with_node(&nodes)
    }
}
