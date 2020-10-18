use wasm_bindgen::JsValue;

pub fn window() -> web_sys::Window {
    web_sys::window().expect("Expected there to be a global `window`.")
}

pub fn document() -> web_sys::Document {
    window().document().expect("should have a document")
}

/// An error in ascessing an object's property.
#[derive(Debug, Clone, PartialEq)]
pub enum GetObjectPropertyError {
    /// Failed to access the value
    AccessFailed {
        /// property-name
        key: &'static str,
    },
    /// We failed to cast to the type
    IncorrectType {
        /// property-name
        key: &'static str,
        /// name of the type as given to us by rust
        type_name: &'static str,
    },
}
impl Into<String> for GetObjectPropertyError {
    fn into(self) -> String {
        match self {
            Self::AccessFailed { key } => format!("Failed to get property: '{}'", key),
            Self::IncorrectType { key, type_name } => {
                format!("Expected property '{}' to be of type '{}'", key, type_name)
            }
        }
    }
}
impl From<GetObjectPropertyError> for JsValue {
    fn from(error: GetObjectPropertyError) -> JsValue {
        let error_message: String = error.into();
        JsValue::from_str(error_message.as_str())
    }
}
/// Get a property of a javascript object as a specific type.
/// Returns an error if the property does not exist, or if fails to cast it into the
/// requested type
pub fn get_object_property<T>(
    object: &JsValue,
    key: &'static str,
) -> Result<T, GetObjectPropertyError>
where
    T: wasm_bindgen::JsCast,
{
    use wasm_bindgen::JsCast;
    js_sys::Reflect::get(object, &JsValue::from_str(key))
        .map_err(|_| GetObjectPropertyError::AccessFailed { key })
        .and_then(|x| {
            x.dyn_into::<T>()
                .map_err(|_| GetObjectPropertyError::IncorrectType {
                    key,
                    type_name: std::any::type_name::<T>(),
                })
        })
}
