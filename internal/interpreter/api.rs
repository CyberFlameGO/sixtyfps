// Copyright © SixtyFPS GmbH <info@slint-ui.com>
// SPDX-License-Identifier: GPL-3.0-only OR LicenseRef-Slint-commercial

use core::convert::TryInto;
use i_slint_compiler::langtype::Type as LangType;
use i_slint_core::graphics::Image;
use i_slint_core::model::{Model, ModelRc};
use i_slint_core::{Brush, PathData, SharedString, SharedVector};
use std::borrow::Cow;
use std::collections::HashMap;
use std::iter::FromIterator;
use std::path::{Path, PathBuf};
use std::rc::Rc;

#[doc(inline)]
pub use i_slint_compiler::diagnostics::{Diagnostic, DiagnosticLevel};

pub use i_slint_core::api::*;

use crate::dynamic_component::ErasedComponentBox;

/// This enum represents the different public variants of the [`Value`] enum, without
/// the contained values.
#[derive(Debug, Copy, Clone, PartialEq)]
#[repr(i8)]
#[non_exhaustive]
pub enum ValueType {
    /// The variant that expresses the non-type. This is the default.
    Void,
    /// An `int` or a `float` (this is also used for unit based type such as `length` or `angle`)
    Number,
    /// Correspond to the `string` type in .slint
    String,
    /// Correspond to the `bool` type in .slint
    Bool,
    /// A model (that includes array in .slint)
    Model,
    /// An object
    Struct,
    /// Correspond to `brush` or `color` type in .slint.  For color, this is then a [`Brush::SolidColor`]
    Brush,
    /// Correspond to `image` type in .slint.
    Image,
    /// The type is not a public type but something internal.
    #[doc(hidden)]
    Other = -1,
}

impl From<LangType> for ValueType {
    fn from(ty: LangType) -> Self {
        match ty {
            LangType::Float32
            | LangType::Int32
            | LangType::Duration
            | LangType::Angle
            | LangType::PhysicalLength
            | LangType::LogicalLength
            | LangType::Percent
            | LangType::UnitProduct(_) => Self::Number,
            LangType::String => Self::String,
            LangType::Color => Self::Brush,
            LangType::Array(_) => Self::Model,
            LangType::Bool => Self::Bool,
            LangType::Struct { .. } => Self::Struct,
            LangType::Void => Self::Void,
            LangType::Image => Self::Image,
            _ => Self::Other,
        }
    }
}

/// This is a dynamically typed value used in the Slint interpreter.
/// It can hold a value of different types, and you should use the
/// [`From`] or [`TryInto`] traits to access the value.
///
/// ```
/// # use slint_interpreter::*;
/// use core::convert::TryInto;
/// // create a value containing an integer
/// let v = Value::from(100u32);
/// assert_eq!(v.try_into(), Ok(100u32));
/// ```
#[derive(Clone)]
#[non_exhaustive]
#[repr(C)]
pub enum Value {
    /// There is nothing in this value. That's the default.
    /// For example, a function that do not return a result would return a Value::Void
    Void,
    /// An `int` or a `float` (this is also used for unit based type such as `length` or `angle`)
    Number(f64),
    /// Correspond to the `string` type in .slint
    String(SharedString),
    /// Correspond to the `bool` type in .slint
    Bool(bool),
    /// Correspond to the `image` type in .slint
    Image(Image),
    /// A model (that includes array in .slint)
    Model(ModelRc<Value>),
    /// An object
    Struct(Struct),
    /// Correspond to `brush` or `color` type in .slint.  For color, this is then a [`Brush::SolidColor`]
    Brush(Brush),
    #[doc(hidden)]
    /// The elements of a path
    PathData(PathData),
    #[doc(hidden)]
    /// An easing curve
    EasingCurve(i_slint_core::animations::EasingCurve),
    #[doc(hidden)]
    /// An enumeration, like `TextHorizontalAlignment::align_center`, represented by `("TextHorizontalAlignment", "align_center")`.
    /// FIXME: consider representing that with a number?
    EnumerationValue(String, String),
    #[doc(hidden)]
    LayoutCache(SharedVector<f32>),
}

impl Value {
    /// Returns the type variant that this value holds without the containing value.
    pub fn value_type(&self) -> ValueType {
        match self {
            Value::Void => ValueType::Void,
            Value::Number(_) => ValueType::Number,
            Value::String(_) => ValueType::String,
            Value::Bool(_) => ValueType::Bool,
            Value::Model(_) => ValueType::Model,
            Value::Struct(_) => ValueType::Struct,
            Value::Brush(_) => ValueType::Brush,
            Value::Image(_) => ValueType::Image,
            _ => ValueType::Other,
        }
    }
}

impl Default for Value {
    fn default() -> Self {
        Value::Void
    }
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match self {
            Value::Void => matches!(other, Value::Void),
            Value::Number(lhs) => matches!(other, Value::Number(rhs) if lhs == rhs),
            Value::String(lhs) => matches!(other, Value::String(rhs) if lhs == rhs),
            Value::Bool(lhs) => matches!(other, Value::Bool(rhs) if lhs == rhs),
            Value::Image(lhs) => matches!(other, Value::Image(rhs) if lhs == rhs),
            Value::Model(lhs) => {
                if let Value::Model(rhs) = other {
                    lhs == rhs
                } else {
                    false
                }
            }
            Value::Struct(lhs) => matches!(other, Value::Struct(rhs) if lhs == rhs),
            Value::Brush(lhs) => matches!(other, Value::Brush(rhs) if lhs == rhs),
            Value::PathData(lhs) => matches!(other, Value::PathData(rhs) if lhs == rhs),
            Value::EasingCurve(lhs) => matches!(other, Value::EasingCurve(rhs) if lhs == rhs),
            Value::EnumerationValue(lhs_name, lhs_value) => {
                matches!(other, Value::EnumerationValue(rhs_name, rhs_value) if lhs_name == rhs_name && lhs_value == rhs_value)
            }
            Value::LayoutCache(lhs) => matches!(other, Value::LayoutCache(rhs) if lhs == rhs),
        }
    }
}

impl std::fmt::Debug for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Void => write!(f, "Value::Void"),
            Value::Number(n) => write!(f, "Value::Number({:?})", n),
            Value::String(s) => write!(f, "Value::String({:?})", s),
            Value::Bool(b) => write!(f, "Value::Bool({:?})", b),
            Value::Image(i) => write!(f, "Value::Image({:?})", i),
            Value::Model(m) => {
                write!(f, "Value::Model(")?;
                f.debug_list().entries(m.iter()).finish()?;
                write!(f, "])")
            }
            Value::Struct(s) => write!(f, "Value::Struct({:?})", s),
            Value::Brush(b) => write!(f, "Value::Brush({:?})", b),
            Value::PathData(e) => write!(f, "Value::PathElements({:?})", e),
            Value::EasingCurve(c) => write!(f, "Value::EasingCurve({:?})", c),
            Value::EnumerationValue(n, v) => write!(f, "Value::EnumerationValue({:?}, {:?})", n, v),
            Value::LayoutCache(v) => write!(f, "Value::LayoutCache({:?})", v),
        }
    }
}

/// Helper macro to implement the From / TryInto for Value
///
/// For example
/// `declare_value_conversion!(Number => [u32, u64, i32, i64, f32, f64] );`
/// means that `Value::Number` can be converted to / from each of the said rust types
///
/// For `Value::Object` mapping to a rust `struct`, one can use [`declare_value_struct_conversion!`]
/// And for `Value::EnumerationValue` which maps to a rust `enum`, one can use [`declare_value_struct_conversion!`]
macro_rules! declare_value_conversion {
    ( $value:ident => [$($ty:ty),*] ) => {
        $(
            impl From<$ty> for Value {
                fn from(v: $ty) -> Self {
                    Value::$value(v as _)
                }
            }
            impl TryInto<$ty> for Value {
                type Error = Value;
                fn try_into(self) -> Result<$ty, Value> {
                    match self {
                        //Self::$value(x) => x.try_into().map_err(|_|()),
                        Self::$value(x) => Ok(x as _),
                        _ => Err(self)
                    }
                }
            }
        )*
    };
}
declare_value_conversion!(Number => [u32, u64, i32, i64, f32, f64, usize, isize] );
declare_value_conversion!(String => [SharedString] );
declare_value_conversion!(Bool => [bool] );
declare_value_conversion!(Image => [Image] );
declare_value_conversion!(Struct => [Struct] );
declare_value_conversion!(Brush => [Brush] );
declare_value_conversion!(PathData => [PathData]);
declare_value_conversion!(EasingCurve => [i_slint_core::animations::EasingCurve]);
declare_value_conversion!(LayoutCache => [SharedVector<f32>] );

/// Implement From / TryInto for Value that convert a `struct` to/from `Value::Object`
macro_rules! declare_value_struct_conversion {
    (struct $name:path { $($field:ident),* $(, ..$extra:expr)? }) => {
        impl From<$name> for Value {
            fn from($name { $($field),* , .. }: $name) -> Self {
                let mut struct_ = Struct::default();
                $(struct_.set_field(stringify!($field).into(), $field.into());)*
                Value::Struct(struct_)
            }
        }
        impl TryInto<$name> for Value {
            type Error = ();
            fn try_into(self) -> Result<$name, ()> {
                match self {
                    Self::Struct(x) => {
                        type Ty = $name;
                        Ok(Ty {
                            $($field: x.get_field(stringify!($field)).ok_or(())?.clone().try_into().map_err(|_|())?),*
                            $(, ..$extra)?
                        })
                    }
                    _ => Err(()),
                }
            }
        }
    };
}

declare_value_struct_conversion!(struct i_slint_core::model::StandardListViewItem { text });
declare_value_struct_conversion!(struct i_slint_core::properties::StateInfo { current_state, previous_state, change_time });
declare_value_struct_conversion!(struct i_slint_core::input::KeyboardModifiers { control, alt, shift, meta });
declare_value_struct_conversion!(struct i_slint_core::input::KeyEvent { event_type, text, modifiers });
declare_value_struct_conversion!(struct i_slint_core::layout::LayoutInfo { min, max, min_percent, max_percent, preferred, stretch });
declare_value_struct_conversion!(struct i_slint_core::graphics::Point { x, y, ..Default::default()});
declare_value_struct_conversion!(struct i_slint_core::items::PointerEvent { kind, button });

/// Implement From / TryInto for Value that convert an `enum` to/from `Value::EnumerationValue`
///
/// The `enum` must derive `Display` and `FromStr`
/// (can be done with `strum_macros::EnumString`, `strum_macros::Display` derive macro)
macro_rules! declare_value_enum_conversion {
    ($ty:ty, $n:ident) => {
        impl From<$ty> for Value {
            fn from(v: $ty) -> Self {
                Value::EnumerationValue(stringify!($n).to_owned(), v.to_string().replace('_', "-"))
            }
        }
        impl TryInto<$ty> for Value {
            type Error = ();
            fn try_into(self) -> Result<$ty, ()> {
                use std::str::FromStr;
                match self {
                    Self::EnumerationValue(enumeration, value) => {
                        if enumeration != stringify!($n) {
                            return Err(());
                        }

                        <$ty>::from_str(value.as_str())
                            .or_else(|_| <$ty>::from_str(&value.as_str().replace('-', "_")))
                            .map_err(|_| ())
                    }
                    _ => Err(()),
                }
            }
        }
    };
}

declare_value_enum_conversion!(
    i_slint_core::items::TextHorizontalAlignment,
    TextHorizontalAlignment
);
declare_value_enum_conversion!(i_slint_core::items::TextVerticalAlignment, TextVerticalAlignment);
declare_value_enum_conversion!(i_slint_core::items::TextOverflow, TextOverflow);
declare_value_enum_conversion!(i_slint_core::items::TextWrap, TextWrap);
declare_value_enum_conversion!(i_slint_core::layout::LayoutAlignment, LayoutAlignment);
declare_value_enum_conversion!(i_slint_core::items::ImageFit, ImageFit);
declare_value_enum_conversion!(i_slint_core::items::ImageRendering, ImageRendering);
declare_value_enum_conversion!(i_slint_core::input::KeyEventType, KeyEventType);
declare_value_enum_conversion!(i_slint_core::items::EventResult, EventResult);
declare_value_enum_conversion!(i_slint_core::items::FillRule, FillRule);
declare_value_enum_conversion!(i_slint_core::items::MouseCursor, MouseCursor);
declare_value_enum_conversion!(i_slint_core::items::StandardButtonKind, StandardButtonKind);
declare_value_enum_conversion!(i_slint_core::items::PointerEventKind, PointerEventKind);
declare_value_enum_conversion!(i_slint_core::items::PointerEventButton, PointerEventButton);
declare_value_enum_conversion!(i_slint_core::items::DialogButtonRole, DialogButtonRole);
declare_value_enum_conversion!(i_slint_core::graphics::PathEvent, PathEvent);

impl From<i_slint_core::animations::Instant> for Value {
    fn from(value: i_slint_core::animations::Instant) -> Self {
        Value::Number(value.0 as _)
    }
}
impl TryInto<i_slint_core::animations::Instant> for Value {
    type Error = ();
    fn try_into(self) -> Result<i_slint_core::animations::Instant, ()> {
        match self {
            Value::Number(x) => Ok(i_slint_core::animations::Instant(x as _)),
            _ => Err(()),
        }
    }
}

impl From<()> for Value {
    #[inline]
    fn from(_: ()) -> Self {
        Value::Void
    }
}
impl TryInto<()> for Value {
    type Error = ();
    #[inline]
    fn try_into(self) -> Result<(), ()> {
        Ok(())
    }
}

impl From<i_slint_core::Color> for Value {
    #[inline]
    fn from(c: i_slint_core::Color) -> Self {
        Value::Brush(Brush::SolidColor(c))
    }
}
impl TryInto<i_slint_core::Color> for Value {
    type Error = Value;
    #[inline]
    fn try_into(self) -> Result<i_slint_core::Color, Value> {
        match self {
            Value::Brush(Brush::SolidColor(c)) => Ok(c),
            _ => Err(self),
        }
    }
}

/// Normalize the identifier to use dashes
pub(crate) fn normalize_identifier(ident: &str) -> Cow<'_, str> {
    if ident.contains('_') {
        ident.replace('_', "-").into()
    } else {
        ident.into()
    }
}

/// This type represents a runtime instance of structure in `.slint`.
///
/// This can either be an instance of a name structure introduced
/// with the `struct` keyword in the .slint file, or an anonymous struct
/// written with the `{ key: value, }`  notation.
///
/// It can be constructed with the [`FromIterator`] trait, and converted
/// into or from a [`Value`] with the [`From`] and [`TryInto`] trait
///
///
/// ```
/// # use slint_interpreter::*;
/// use core::convert::TryInto;
/// // Construct a value from a key/value iterator
/// let value : Value = [("foo".into(), 45u32.into()), ("bar".into(), true.into())]
///     .iter().cloned().collect::<Struct>().into();
///
/// // get the properties of a `{ foo: 45, bar: true }`
/// let s : Struct = value.try_into().unwrap();
/// assert_eq!(s.get_field("foo").cloned().unwrap().try_into(), Ok(45u32));
/// ```
#[derive(Clone, PartialEq, Debug, Default)]
pub struct Struct(HashMap<String, Value>);
impl Struct {
    /// Get the value for a given struct field
    pub fn get_field(&self, name: &str) -> Option<&Value> {
        self.0.get(&*normalize_identifier(name))
    }
    /// Set the value of a given struct field
    pub fn set_field(&mut self, name: String, value: Value) {
        if name.contains('_') {
            self.0.insert(name.replace('_', "-"), value);
        } else {
            self.0.insert(name, value);
        }
    }

    /// Iterate over all the fields in this struct
    pub fn iter(&self) -> impl Iterator<Item = (&str, &Value)> {
        self.0.iter().map(|(a, b)| (a.as_str(), b))
    }
}

impl FromIterator<(String, Value)> for Struct {
    fn from_iter<T: IntoIterator<Item = (String, Value)>>(iter: T) -> Self {
        Self(
            iter.into_iter()
                .map(|(s, v)| (if s.contains('_') { s.replace('_', "-") } else { s }, v))
                .collect(),
        )
    }
}

/// ComponentCompiler is the entry point to the Slint interpreter that can be used
/// to load .slint files or compile them on-the-fly from a string.
pub struct ComponentCompiler {
    config: i_slint_compiler::CompilerConfiguration,
    diagnostics: Vec<Diagnostic>,
}

impl Default for ComponentCompiler {
    fn default() -> Self {
        Self {
            config: i_slint_compiler::CompilerConfiguration::new(
                i_slint_compiler::generator::OutputFormat::Interpreter,
            ),
            diagnostics: vec![],
        }
    }
}

impl ComponentCompiler {
    /// Returns a new ComponentCompiler.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the include paths used for looking up `.slint` imports to the specified vector of paths.
    pub fn set_include_paths(&mut self, include_paths: Vec<std::path::PathBuf>) {
        self.config.include_paths = include_paths;
    }

    /// Returns the include paths the component compiler is currently configured with.
    pub fn include_paths(&self) -> &Vec<std::path::PathBuf> {
        &self.config.include_paths
    }

    /// Sets the style to be used for widgets.
    pub fn set_style(&mut self, style: String) {
        self.config.style = Some(style);
    }

    /// Returns the widget style the compiler is currently using when compiling .slint files.
    pub fn style(&self) -> Option<&String> {
        self.config.style.as_ref()
    }

    /// Sets the callback that will be invoked when loading imported .slint files. The specified
    /// `file_loader_callback` parameter will be called with a canonical file path as argument
    /// and is expected to return a future that, when resolved, provides the source code of the
    /// .slint file to be imported as a string.
    /// If an error is returned, then the build will abort with that error.
    /// If None is returned, it means the normal resolution algorithm will proceed as if the hook
    /// was not in place (i.e: load from the file system following the include paths)
    pub fn set_file_loader(
        &mut self,
        file_loader_fallback: impl Fn(
                &Path,
            ) -> core::pin::Pin<
                Box<dyn core::future::Future<Output = Option<std::io::Result<String>>>>,
            > + 'static,
    ) {
        self.config.open_import_fallback =
            Some(Rc::new(move |path| file_loader_fallback(Path::new(path.as_str()))));
    }

    /// Returns the diagnostics that were produced in the last call to [`Self::build_from_path`] or [`Self::build_from_source`].
    pub fn diagnostics(&self) -> &Vec<Diagnostic> {
        &self.diagnostics
    }

    /// Compile a .slint file into a ComponentDefinition
    ///
    /// Returns the compiled `ComponentDefinition` if there were no errors.
    ///
    /// Any diagnostics produced during the compilation, such as warnings or errors, are collected
    /// in this ComponentCompiler and can be retrieved after the call using the [`Self::diagnostics()`]
    /// function. The [`print_diagnostics`] function can be used to display the diagnostics
    /// to the users.
    ///
    /// Diagnostics from previous calls are cleared when calling this function.
    ///
    /// If the path is `"-"`, the file will be read from stdin.
    ///
    /// This function is `async` but in practice, this is only asynchronous if
    /// [`Self::set_file_loader`] was called and its future is actually asynchronous.
    /// If that is not used, then it is fine to use a very simple executor, such as the one
    /// provided by the `spin_on` crate
    pub async fn build_from_path<P: AsRef<Path>>(
        &mut self,
        path: P,
    ) -> Option<ComponentDefinition> {
        let path = path.as_ref();
        let source = match i_slint_compiler::diagnostics::load_from_path(path) {
            Ok(s) => s,
            Err(d) => {
                self.diagnostics = vec![d];
                return None;
            }
        };

        generativity::make_guard!(guard);
        let (c, diag) =
            crate::dynamic_component::load(source, path.into(), self.config.clone(), guard).await;
        self.diagnostics = diag.into_iter().collect();
        c.ok().map(|inner| ComponentDefinition { inner: inner.into() })
    }

    /// Compile some .slint code into a ComponentDefinition
    ///
    /// The `path` argument will be used for diagnostics and to compute relative
    /// paths while importing.
    ///
    /// Any diagnostics produced during the compilation, such as warnings or errors, are collected
    /// in this ComponentCompiler and can be retrieved after the call using the [`Self::diagnostics()`]
    /// function. The [`print_diagnostics`] function can be used to display the diagnostics
    /// to the users.
    ///
    /// Diagnostics from previous calls are cleared when calling this function.
    ///
    /// This function is `async` but in practice, this is only asynchronous if
    /// [`Self::set_file_loader`] is set and its future is actually asynchronous.
    /// If that is not used, then it is fine to use a very simple executor, such as the one
    /// provided by the `spin_on` crate
    pub async fn build_from_source(
        &mut self,
        source_code: String,
        path: PathBuf,
    ) -> Option<ComponentDefinition> {
        generativity::make_guard!(guard);
        let (c, diag) =
            crate::dynamic_component::load(source_code, path, self.config.clone(), guard).await;
        self.diagnostics = diag.into_iter().collect();
        c.ok().map(|inner| ComponentDefinition { inner: inner.into() })
    }
}

/// ComponentDefinition is a representation of a compiled component from .slint markup.
///
/// It can be constructed from a .slint file using the [`ComponentCompiler::build_from_path`] or [`ComponentCompiler::build_from_source`] functions.
/// And then it can be instantiated with the [`Self::create`] function.
///
/// The ComponentDefinition acts as a factory to create new instances. When you've finished
/// creating the instances it is safe to drop the ComponentDefinition.
#[derive(Clone)]
pub struct ComponentDefinition {
    inner: crate::dynamic_component::ErasedComponentDescription,
}

impl ComponentDefinition {
    /// Creates a new instance of the component and returns a shared handle to it.
    pub fn create(&self) -> ComponentInstance {
        generativity::make_guard!(guard);
        ComponentInstance {
            inner: self.inner.unerase(guard).clone().create(
                #[cfg(target_arch = "wasm32")]
                "canvas".into(),
            ),
        }
    }

    /// Instantiate the component for wasm using the given canvas id
    #[cfg(target_arch = "wasm32")]
    pub fn create_with_canvas_id(&self, canvas_id: &str) -> ComponentInstance {
        generativity::make_guard!(guard);
        ComponentInstance { inner: self.inner.unerase(guard).clone().create(canvas_id.into()) }
    }

    /// Instantiate the component using an existing window.
    /// This method is internal because the WindowRc is not a public type
    #[doc(hidden)]
    pub fn create_with_existing_window(&self, window: &Window) -> ComponentInstance {
        use i_slint_core::window::WindowHandleAccess;
        generativity::make_guard!(guard);
        ComponentInstance {
            inner: self
                .inner
                .unerase(guard)
                .clone()
                .create_with_existing_window(window.window_handle()),
        }
    }

    /// List of publicly declared properties or callback.
    ///
    /// This is internal because it exposes the `Type` from compilerlib.
    #[doc(hidden)]
    pub fn properties_and_callbacks(
        &self,
    ) -> impl Iterator<Item = (String, i_slint_compiler::langtype::Type)> + '_ {
        // We create here a 'static guard, because unfortunately the returned type would be restricted to the guard lifetime
        // which is not required, but this is safe because there is only one instance of the unerased type
        let guard = unsafe { generativity::Guard::new(generativity::Id::new()) };
        self.inner.unerase(guard).properties()
    }

    /// List of publicly declared properties.
    pub fn properties(&self) -> impl Iterator<Item = (String, ValueType)> + '_ {
        // We create here a 'static guard, because unfortunately the returned type would be restricted to the guard lifetime
        // which is not required, but this is safe because there is only one instance of the unerased type
        let guard = unsafe { generativity::Guard::new(generativity::Id::new()) };
        self.inner.unerase(guard).properties().filter_map(|(prop_name, prop_type)| {
            if prop_type.is_property_type() {
                Some((prop_name, prop_type.into()))
            } else {
                None
            }
        })
    }

    /// Returns the names of all publicly declared callbacks.
    pub fn callbacks(&self) -> impl Iterator<Item = String> + '_ {
        // We create here a 'static guard, because unfortunately the returned type would be restricted to the guard lifetime
        // which is not required, but this is safe because there is only one instance of the unerased type
        let guard = unsafe { generativity::Guard::new(generativity::Id::new()) };
        self.inner.unerase(guard).properties().filter_map(|(prop_name, prop_type)| {
            if matches!(prop_type, LangType::Callback { .. }) {
                Some(prop_name)
            } else {
                None
            }
        })
    }

    /// Returns the names of all exported global singletons
    ///
    /// **Note:** Only globals that are exported or re-exported from the main .slint file will
    /// be exposed in the API
    pub fn globals(&self) -> impl Iterator<Item = String> + '_ {
        // We create here a 'static guard, because unfortunately the returned type would be restricted to the guard lifetime
        // which is not required, but this is safe because there is only one instance of the unerased type
        let guard = unsafe { generativity::Guard::new(generativity::Id::new()) };
        self.inner.unerase(guard).global_names()
    }

    /// List of publicly declared properties in the exported global singleton specified by its name.
    pub fn global_properties(
        &self,
        global_name: &str,
    ) -> Option<impl Iterator<Item = (String, ValueType)> + '_> {
        // We create here a 'static guard, because unfortunately the returned type would be restricted to the guard lifetime
        // which is not required, but this is safe because there is only one instance of the unerased type
        let guard = unsafe { generativity::Guard::new(generativity::Id::new()) };
        self.inner.unerase(guard).global_properties(global_name).map(|iter| {
            iter.filter_map(|(prop_name, prop_type)| {
                if prop_type.is_property_type() {
                    Some((prop_name, prop_type.into()))
                } else {
                    None
                }
            })
        })
    }

    /// List of publicly declared callbacks in the exported global singleton specified by its name.
    pub fn global_callbacks(&self, global_name: &str) -> Option<impl Iterator<Item = String> + '_> {
        // We create here a 'static guard, because unfortunately the returned type would be restricted to the guard lifetime
        // which is not required, but this is safe because there is only one instance of the unerased type
        let guard = unsafe { generativity::Guard::new(generativity::Id::new()) };
        self.inner.unerase(guard).global_properties(global_name).map(|iter| {
            iter.filter_map(|(prop_name, prop_type)| {
                if matches!(prop_type, LangType::Callback { .. }) {
                    Some(prop_name)
                } else {
                    None
                }
            })
        })
    }

    /// The name of this Component as written in the .slint file
    pub fn name(&self) -> &str {
        // We create here a 'static guard, because unfortunately the returned type would be restricted to the guard lifetime
        // which is not required, but this is safe because there is only one instance of the unerased type
        let guard = unsafe { generativity::Guard::new(generativity::Id::new()) };
        self.inner.unerase(guard).id()
    }
}

/// Print the diagnostics to stderr
///
/// The diagnostics are printed in the same style as rustc errors
///
/// This function is available when the `display-diagnostics` is enabled.
#[cfg(feature = "display-diagnostics")]
pub fn print_diagnostics(diagnostics: &[Diagnostic]) {
    let mut build_diagnostics = i_slint_compiler::diagnostics::BuildDiagnostics::default();
    for d in diagnostics {
        build_diagnostics.push_compiler_error(d.clone())
    }
    build_diagnostics.print();
}

/// This represent an instance of a dynamic component
///
/// You can create an instance with the [`ComponentDefinition::create`] function.
///
/// Properties and callback can be accessed using the associated functions.
///
/// An instance can be put on screen with the [`ComponentInstance::run`] function.
#[repr(C)]
pub struct ComponentInstance {
    inner: vtable::VRc<
        i_slint_core::component::ComponentVTable,
        crate::dynamic_component::ErasedComponentBox,
    >,
}

impl ComponentInstance {
    /// Return the [`ComponentDefinition`] that was used to create this instance.
    pub fn definition(&self) -> ComponentDefinition {
        generativity::make_guard!(guard);
        ComponentDefinition { inner: self.inner.unerase(guard).description().into() }
    }

    /// Return the value for a public property of this component.
    ///
    /// ## Examples
    ///
    /// ```
    /// use slint_interpreter::{ComponentDefinition, ComponentCompiler, Value, SharedString};
    /// let code = r#"
    ///     MyWin := Window {
    ///         property <int> my_property: 42;
    ///     }
    /// "#;
    /// let mut compiler = ComponentCompiler::default();
    /// let definition = spin_on::spin_on(
    ///     compiler.build_from_source(code.into(), Default::default()));
    /// assert!(compiler.diagnostics().is_empty(), "{:?}", compiler.diagnostics());
    /// let instance = definition.unwrap().create();
    /// assert_eq!(instance.get_property("my_property").unwrap(), Value::from(42));
    /// ```
    pub fn get_property(&self, name: &str) -> Result<Value, GetPropertyError> {
        generativity::make_guard!(guard);
        let comp = self.inner.unerase(guard);
        comp.description()
            .get_property(comp.borrow(), &normalize_identifier(name))
            .map_err(|()| GetPropertyError::NoSuchProperty)
    }

    /// Set the value for a public property of this component
    pub fn set_property(&self, name: &str, value: Value) -> Result<(), SetPropertyError> {
        generativity::make_guard!(guard);
        let comp = self.inner.unerase(guard);
        comp.description().set_property(comp.borrow(), &normalize_identifier(name), value)
    }

    /// Set a handler for the callback with the given name. A callback with that
    /// name must be defined in the document otherwise an error will be returned.
    ///
    /// Note: Since the [`ComponentInstance`] holds the handler, the handler itself should not
    /// contain a strong reference to the instance. So if you need to capture the instance,
    /// you should use [`Self::as_weak`] to create a weak reference.
    ///
    /// ## Examples
    ///
    /// ```
    /// use slint_interpreter::{ComponentDefinition, ComponentCompiler, Value, SharedString, ComponentHandle};
    /// use core::convert::TryInto;
    /// let code = r#"
    ///     MyWin := Window {
    ///         callback foo(int) -> int;
    ///         property <int> my_prop: 12;
    ///     }
    /// "#;
    /// let definition = spin_on::spin_on(
    ///     ComponentCompiler::default().build_from_source(code.into(), Default::default()));
    /// let instance = definition.unwrap().create();
    ///
    /// let instance_weak = instance.as_weak();
    /// instance.set_callback("foo", move |args: &[Value]| -> Value {
    ///     let arg: u32 = args[0].clone().try_into().unwrap();
    ///     let my_prop = instance_weak.unwrap().get_property("my_prop").unwrap();
    ///     let my_prop : u32 = my_prop.try_into().unwrap();
    ///     Value::from(arg + my_prop)
    /// }).unwrap();
    ///
    /// let res = instance.invoke_callback("foo", &[Value::from(500)]).unwrap();
    /// assert_eq!(res, Value::from(500+12));
    /// ```
    pub fn set_callback(
        &self,
        name: &str,
        callback: impl Fn(&[Value]) -> Value + 'static,
    ) -> Result<(), SetCallbackError> {
        generativity::make_guard!(guard);
        let comp = self.inner.unerase(guard);
        comp.description()
            .set_callback_handler(comp.borrow(), &normalize_identifier(name), Box::new(callback))
            .map_err(|()| SetCallbackError::NoSuchCallback)
    }

    /// Call the given callback with the arguments
    ///
    /// ## Examples
    /// See the documentation of [`Self::set_callback`] for an example
    pub fn invoke_callback(
        &self,
        name: &str,
        args: &[Value],
    ) -> Result<Value, InvokeCallbackError> {
        generativity::make_guard!(guard);
        let comp = self.inner.unerase(guard);
        comp.description()
            .invoke_callback(comp.borrow(), &normalize_identifier(name), args)
            .map_err(|()| InvokeCallbackError::NoSuchCallback)
    }

    /// Return the value for a property within an exported global singleton used by this component.
    ///
    /// The `global` parameter is the exported name of the global singleton. The `property` argument
    /// is the name of the property
    ///
    /// ## Examples
    ///
    /// ```
    /// use slint_interpreter::{ComponentDefinition, ComponentCompiler, Value, SharedString};
    /// let code = r#"
    ///     global Glob := {
    ///         property <int> my_property: 42;
    ///     }
    ///     export { Glob as TheGlobal }
    ///     MyWin := Window {
    ///     }
    /// "#;
    /// let mut compiler = ComponentCompiler::default();
    /// let definition = spin_on::spin_on(
    ///     compiler.build_from_source(code.into(), Default::default()));
    /// assert!(compiler.diagnostics().is_empty(), "{:?}", compiler.diagnostics());
    /// let instance = definition.unwrap().create();
    /// assert_eq!(instance.get_global_property("TheGlobal", "my_property").unwrap(), Value::from(42));
    /// ```
    pub fn get_global_property(
        &self,
        global: &str,
        property: &str,
    ) -> Result<Value, GetPropertyError> {
        generativity::make_guard!(guard);
        let comp = self.inner.unerase(guard);
        comp.description()
            .get_global(comp.borrow(), &normalize_identifier(global))
            .map_err(|()| GetPropertyError::NoSuchProperty)? // FIXME: should there be a NoSuchGlobal error?
            .as_ref()
            .get_property(&normalize_identifier(property))
            .map_err(|()| GetPropertyError::NoSuchProperty)
    }

    /// Set the value for a property within an exported global singleton used by this component.
    pub fn set_global_property(
        &self,
        global: &str,
        property: &str,
        value: Value,
    ) -> Result<(), SetPropertyError> {
        generativity::make_guard!(guard);
        let comp = self.inner.unerase(guard);
        comp.description()
            .get_global(comp.borrow(), &normalize_identifier(global))
            .map_err(|()| SetPropertyError::NoSuchProperty)? // FIXME: should there be a NoSuchGlobal error?
            .as_ref()
            .set_property(&normalize_identifier(property), value)
    }

    /// Set a handler for the callback in the exported global singleton. A callback with that
    /// name must be defined in the specified global and the global must be exported from the
    /// main document otherwise an error will be returned.
    ///
    /// ## Examples
    ///
    /// ```
    /// use slint_interpreter::{ComponentDefinition, ComponentCompiler, Value, SharedString};
    /// use core::convert::TryInto;
    /// let code = r#"
    ///     export global Logic := {
    ///         callback to_uppercase(string) -> string;
    ///     }
    ///     MyWin := Window {
    ///         property <string> hello: Logic.to_uppercase("world");
    ///     }
    /// "#;
    /// let definition = spin_on::spin_on(
    ///     ComponentCompiler::default().build_from_source(code.into(), Default::default()));
    /// let instance = definition.unwrap().create();
    /// instance.set_global_callback("Logic", "to_uppercase", |args: &[Value]| -> Value {
    ///     let arg: SharedString = args[0].clone().try_into().unwrap();
    ///     Value::from(SharedString::from(arg.to_uppercase()))
    /// }).unwrap();
    ///
    /// let res = instance.get_property("hello").unwrap();
    /// assert_eq!(res, Value::from(SharedString::from("WORLD")));
    ///
    /// let abc = instance.invoke_global_callback("Logic", "to_uppercase", &[
    ///     SharedString::from("abc").into()
    /// ]).unwrap();
    /// assert_eq!(abc, Value::from(SharedString::from("ABC")));
    /// ```
    pub fn set_global_callback(
        &self,
        global: &str,
        name: &str,
        callback: impl Fn(&[Value]) -> Value + 'static,
    ) -> Result<(), SetCallbackError> {
        generativity::make_guard!(guard);
        let comp = self.inner.unerase(guard);
        comp.description()
            .get_global(comp.borrow(), &normalize_identifier(global))
            .map_err(|()| SetCallbackError::NoSuchCallback)? // FIXME: should there be a NoSuchGlobal error?
            .as_ref()
            .set_callback_handler(&normalize_identifier(name), Box::new(callback))
            .map_err(|()| SetCallbackError::NoSuchCallback)
    }

    /// Call the given callback within a global singleton with the arguments
    ///
    /// ## Examples
    /// See the documentation of [`Self::set_global_callback`] for an example
    pub fn invoke_global_callback(
        &self,
        global: &str,
        callback_name: &str,
        args: &[Value],
    ) -> Result<Value, InvokeCallbackError> {
        generativity::make_guard!(guard);
        let comp = self.inner.unerase(guard);
        comp.description()
            .get_global(comp.borrow(), &normalize_identifier(global))
            .map_err(|()| InvokeCallbackError::NoSuchCallback)? // FIXME: should there be a NoSuchGlobal error?
            .as_ref()
            .invoke_callback(&normalize_identifier(callback_name), args)
            .map_err(|()| InvokeCallbackError::NoSuchCallback)
    }
}

impl ComponentHandle for ComponentInstance {
    type Inner = crate::dynamic_component::ErasedComponentBox;

    fn as_weak(&self) -> Weak<Self>
    where
        Self: Sized,
    {
        Weak::new(&self.inner)
    }

    fn clone_strong(&self) -> Self {
        Self { inner: self.inner.clone() }
    }

    fn from_inner(
        inner: vtable::VRc<i_slint_core::component::ComponentVTable, Self::Inner>,
    ) -> Self {
        Self { inner }
    }

    fn show(&self) {
        generativity::make_guard!(guard);
        let comp = self.inner.unerase(guard);
        comp.borrow_instance().window().show();
    }

    fn hide(&self) {
        generativity::make_guard!(guard);
        let comp = self.inner.unerase(guard);
        comp.borrow_instance().window().hide();
    }

    fn run(&self) {
        self.show();
        i_slint_backend_selector::backend()
            .run_event_loop(i_slint_core::backend::EventLoopQuitBehavior::QuitOnLastWindowClosed);
        self.hide();
    }

    fn window(&self) -> &Window {
        self.inner.window()
    }

    fn global<'a, T: Global<'a, Self>>(&'a self) -> T
    where
        Self: Sized,
    {
        unreachable!()
    }
}

impl From<ComponentInstance>
    for vtable::VRc<i_slint_core::component::ComponentVTable, ErasedComponentBox>
{
    fn from(value: ComponentInstance) -> Self {
        value.inner
    }
}

/// Error returned by [`ComponentInstance::get_property`]
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
#[non_exhaustive]
pub enum GetPropertyError {
    /// There is no property with the given name
    #[error("no such property")]
    NoSuchProperty,
}

/// Error returned by [`ComponentInstance::set_property`]
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
#[non_exhaustive]
pub enum SetPropertyError {
    /// There is no property with the given name
    #[error("no such property")]
    NoSuchProperty,
    /// The property exist but does not have a type matching the dynamic value
    #[error("wrong type")]
    WrongType,
}

/// Error returned by [`ComponentInstance::set_callback`]
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
#[non_exhaustive]
pub enum SetCallbackError {
    /// There is no callback with the given name
    #[error("no such callback")]
    NoSuchCallback,
}

/// Error returned by [`ComponentInstance::invoke_callback`]
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
#[non_exhaustive]
pub enum InvokeCallbackError {
    /// There is no callback with the given name
    #[error("no such callback")]
    NoSuchCallback,
}

/// Enters the main event loop. This is necessary in order to receive
/// events from the windowing system in order to render to the screen
/// and react to user input.
pub fn run_event_loop() {
    i_slint_backend_selector::backend()
        .run_event_loop(i_slint_core::backend::EventLoopQuitBehavior::QuitOnLastWindowClosed);
}

/// This module contains a few function use by tests
pub mod testing {
    use super::ComponentHandle;
    use i_slint_core::window::WindowHandleAccess;

    /// Wrapper around [`i_slint_core::tests::slint_send_mouse_click`]
    pub fn send_mouse_click(comp: &super::ComponentInstance, x: f32, y: f32) {
        i_slint_core::tests::slint_send_mouse_click(
            &vtable::VRc::into_dyn(comp.inner.clone()),
            x,
            y,
            comp.window().window_handle(),
        );
    }
    /// Wrapper around [`i_slint_core::tests::send_keyboard_string_sequence`]
    pub fn send_keyboard_string_sequence(
        comp: &super::ComponentInstance,
        string: i_slint_core::SharedString,
    ) {
        i_slint_core::tests::send_keyboard_string_sequence(
            &string,
            Default::default(),
            comp.window().window_handle(),
        );
    }
}

#[test]
fn component_definition_properties() {
    i_slint_backend_testing::init();
    let mut compiler = ComponentCompiler::default();
    compiler.set_style("fluent".into());
    let comp_def = spin_on::spin_on(
        compiler.build_from_source(
            r#"
    export Dummy := Rectangle {
        property <string> test;
        property <int> underscores-and-dashes_preserved: 44;
        callback hello;
    }"#
            .into(),
            "".into(),
        ),
    )
    .unwrap();

    let props = comp_def.properties().collect::<Vec<(_, _)>>();

    assert_eq!(props.len(), 2);
    assert_eq!(props[0].0, "test");
    assert_eq!(props[0].1, ValueType::String);
    assert_eq!(props[1].0, "underscores-and-dashes_preserved");
    assert_eq!(props[1].1, ValueType::Number);

    let instance = comp_def.create();
    assert_eq!(instance.get_property("underscores_and-dashes-preserved"), Ok(Value::Number(44.)));
    assert_eq!(
        instance.get_property("underscoresanddashespreserved"),
        Err(GetPropertyError::NoSuchProperty)
    );
    assert_eq!(
        instance.set_property("underscores-and_dashes-preserved", Value::Number(88.)),
        Ok(())
    );
    assert_eq!(
        instance.set_property("underscoresanddashespreserved", Value::Number(99.)),
        Err(SetPropertyError::NoSuchProperty)
    );
    assert_eq!(
        instance.set_property("underscores-and_dashes-preserved", Value::String("99".into())),
        Err(SetPropertyError::WrongType)
    );
    assert_eq!(instance.get_property("underscores-and-dashes-preserved"), Ok(Value::Number(88.)));
}

#[test]
fn component_definition_properties2() {
    i_slint_backend_testing::init();
    let mut compiler = ComponentCompiler::default();
    compiler.set_style("fluent".into());
    let comp_def = spin_on::spin_on(
        compiler.build_from_source(
            r#"
    export Dummy := Rectangle {
        property <string> sub-text <=> sub.text;
        sub := Text { property <int> private-not-exported; }
        callback hello;
    }"#
            .into(),
            "".into(),
        ),
    )
    .unwrap();

    let props = comp_def.properties().collect::<Vec<(_, _)>>();

    assert_eq!(props.len(), 1);
    assert_eq!(props[0].0, "sub-text");
    assert_eq!(props[0].1, ValueType::String);

    let callbacks = comp_def.callbacks().collect::<Vec<_>>();
    assert_eq!(callbacks.len(), 1);
    assert_eq!(callbacks[0], "hello");
}

#[test]
fn globals() {
    i_slint_backend_testing::init();
    let mut compiler = ComponentCompiler::default();
    compiler.set_style("fluent".into());
    let definition = spin_on::spin_on(
        compiler.build_from_source(
            r#"
    export global My-Super_Global := {
        property <int> the-property : 21;
        callback my-callback();
    }
    export { My-Super_Global as AliasedGlobal }
    export Dummy := Rectangle {
    }"#
            .into(),
            "".into(),
        ),
    )
    .unwrap();

    assert_eq!(definition.globals().collect::<Vec<_>>(), vec!["My-Super_Global", "AliasedGlobal"]);

    assert!(definition.global_properties("not-there").is_none());
    {
        let expected_properties = vec![("the-property".to_string(), ValueType::Number)];
        let expected_callbacks = vec!["my-callback".to_string()];

        let assert_properties_and_callbacks = |global_name| {
            assert_eq!(
                definition
                    .global_properties(global_name)
                    .map(|props| props.collect::<Vec<_>>())
                    .as_ref(),
                Some(&expected_properties)
            );
            assert_eq!(
                definition
                    .global_callbacks(global_name)
                    .map(|props| props.collect::<Vec<_>>())
                    .as_ref(),
                Some(&expected_callbacks)
            );
        };

        assert_properties_and_callbacks("My-Super-Global");
        assert_properties_and_callbacks("My_Super-Global");
        assert_properties_and_callbacks("AliasedGlobal");
    }

    let instance = definition.create();
    assert_eq!(
        instance.set_global_property("My_Super-Global", "the_property", Value::Number(44.)),
        Ok(())
    );
    assert_eq!(
        instance.set_global_property("AliasedGlobal", "the_property", Value::Number(44.)),
        Ok(())
    );
    assert_eq!(
        instance.set_global_property("DontExist", "the-property", Value::Number(88.)),
        Err(SetPropertyError::NoSuchProperty)
    );

    assert_eq!(
        instance.set_global_property("My_Super-Global", "theproperty", Value::Number(88.)),
        Err(SetPropertyError::NoSuchProperty)
    );
    assert_eq!(
        instance.set_global_property("AliasedGlobal", "theproperty", Value::Number(88.)),
        Err(SetPropertyError::NoSuchProperty)
    );
    assert_eq!(
        instance.set_global_property("My_Super-Global", "the_property", Value::String("88".into())),
        Err(SetPropertyError::WrongType)
    );
    assert_eq!(
        instance.get_global_property("My-Super_Global", "yoyo"),
        Err(GetPropertyError::NoSuchProperty)
    );
    assert_eq!(
        instance.get_global_property("My-Super_Global", "the-property"),
        Ok(Value::Number(44.))
    );

    assert_eq!(
        instance.set_property("the-property", Value::Void),
        Err(SetPropertyError::NoSuchProperty)
    );
    assert_eq!(instance.get_property("the-property"), Err(GetPropertyError::NoSuchProperty));

    assert_eq!(
        instance.set_global_callback("DontExist", "the-property", |_| panic!()),
        Err(SetCallbackError::NoSuchCallback)
    );
    assert_eq!(
        instance.set_global_callback("My_Super_Global", "the-property", |_| panic!()),
        Err(SetCallbackError::NoSuchCallback)
    );
    assert_eq!(
        instance.set_global_callback("My_Super_Global", "yoyo", |_| panic!()),
        Err(SetCallbackError::NoSuchCallback)
    );

    assert_eq!(
        instance.invoke_global_callback("DontExist", "the-property", &[]),
        Err(InvokeCallbackError::NoSuchCallback)
    );
    assert_eq!(
        instance.invoke_global_callback("My_Super_Global", "the-property", &[]),
        Err(InvokeCallbackError::NoSuchCallback)
    );
    assert_eq!(
        instance.invoke_global_callback("My_Super_Global", "yoyo", &[]),
        Err(InvokeCallbackError::NoSuchCallback)
    );
}

#[test]
fn component_definition_struct_properties() {
    i_slint_backend_testing::init();
    let mut compiler = ComponentCompiler::default();
    compiler.set_style("fluent".into());
    let comp_def = spin_on::spin_on(
        compiler.build_from_source(
            r#"
    export struct Settings := {
        string_value: string,
    }
    export Dummy := Rectangle {
        property <Settings> test;
    }"#
            .into(),
            "".into(),
        ),
    )
    .unwrap();

    let props = comp_def.properties().collect::<Vec<(_, _)>>();

    assert_eq!(props.len(), 1);
    assert_eq!(props[0].0, "test");
    assert_eq!(props[0].1, ValueType::Struct);

    let instance = comp_def.create();

    let valid_struct: Struct =
        [("string_value".to_string(), Value::String("hello".into()))].iter().cloned().collect();

    assert_eq!(instance.set_property("test", Value::Struct(valid_struct.clone())), Ok(()));
    assert_eq!(instance.get_property("test").unwrap().value_type(), ValueType::Struct);

    assert_eq!(instance.set_property("test", Value::Number(42.)), Err(SetPropertyError::WrongType));

    let mut invalid_struct = valid_struct.clone();
    invalid_struct.set_field("other".into(), Value::Number(44.));
    assert_eq!(
        instance.set_property("test", Value::Struct(invalid_struct)),
        Err(SetPropertyError::WrongType)
    );
    let mut invalid_struct = valid_struct.clone();
    invalid_struct.set_field("string_value".into(), Value::Number(44.));
    assert_eq!(
        instance.set_property("test", Value::Struct(invalid_struct)),
        Err(SetPropertyError::WrongType)
    );
}

#[test]
fn component_definition_model_properties() {
    use i_slint_core::model::*;
    i_slint_backend_testing::init();
    let mut compiler = ComponentCompiler::default();
    compiler.set_style("fluent".into());
    let comp_def = spin_on::spin_on(compiler.build_from_source(
        "export Dummy := Rectangle { property <[int]> prop: [42, 12]; }".into(),
        "".into(),
    ))
    .unwrap();

    let props = comp_def.properties().collect::<Vec<(_, _)>>();
    assert_eq!(props.len(), 1);
    assert_eq!(props[0].0, "prop");
    assert_eq!(props[0].1, ValueType::Model);

    let instance = comp_def.create();

    let int_model = Value::Model(VecModel::from_slice(&[
        Value::Number(14.),
        Value::Number(15.),
        Value::Number(16.),
    ]));
    let empty_model = Value::Model(ModelRc::new(VecModel::<Value>::default()));
    let model_with_string = Value::Model(VecModel::from_slice(&[
        Value::Number(1000.),
        Value::String("foo".into()),
        Value::Number(1111.),
    ]));

    #[track_caller]
    fn check_model(val: Value, r: &[f64]) {
        if let Value::Model(m) = val {
            assert_eq!(r.len(), m.row_count());
            for (i, v) in r.iter().enumerate() {
                assert_eq!(m.row_data(i).unwrap(), Value::Number(*v));
            }
        } else {
            panic!("{:?} not a model", val);
        }
    }

    assert_eq!(instance.get_property("prop").unwrap().value_type(), ValueType::Model);
    check_model(instance.get_property("prop").unwrap(), &[42., 12.]);

    instance.set_property("prop", int_model).unwrap();
    check_model(instance.get_property("prop").unwrap(), &[14., 15., 16.]);

    assert_eq!(instance.set_property("prop", Value::Number(42.)), Err(SetPropertyError::WrongType));
    check_model(instance.get_property("prop").unwrap(), &[14., 15., 16.]);
    assert_eq!(instance.set_property("prop", model_with_string), Err(SetPropertyError::WrongType));
    check_model(instance.get_property("prop").unwrap(), &[14., 15., 16.]);

    assert_eq!(instance.set_property("prop", empty_model), Ok(()));
    check_model(instance.get_property("prop").unwrap(), &[]);
}

#[cfg(feature = "ffi")]
#[allow(missing_docs)]
#[path = "ffi.rs"]
pub(crate) mod ffi;
