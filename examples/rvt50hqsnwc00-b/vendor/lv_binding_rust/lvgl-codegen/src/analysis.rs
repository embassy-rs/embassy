#![allow(dead_code)]

/// A parameter of C functions.
///
/// This struct represents all relevant information we can extract from the C function declaration
/// of a LVGL public interface. We can use this information to do inference for how the parameter
/// should be represented in a safe Rust API.
#[derive(Clone, Debug)]
pub struct CParameter {
    /// The name of the parameter in the C code.
    pub name: String,

    /// This is the raw representation of the Rust equivalent of the C type.
    pub c_type: String,

    /// Takes a pointer to a type that is referenced by the LVGL code permanently.
    pub scope: ParameterScope,

    /// The pointer is not marked as `*const` so the referenced object can be mutated.
    pub mutable: bool,

    /// We need to check if the value is optional in the C code. We need to check
    /// the function comments for this information.
    ///     - "if NULL then"
    ///     - "if not NULL then"
    ///     - "NULL to"
    pub allow_none: bool,

    /// Comment associated with the parameter, if exists.
    pub comment: Option<String>,
}

#[derive(Clone, Debug)]
pub enum ParameterScope {
    Call,
    Static,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FunctionKind {
    Constructor,
    Method,
    Function,
}

/// Inference from a LVGL C API function.
#[derive(Clone, Debug)]
pub struct Function {
    /// Name of the function in the LVGL C API.
    pub name: String,

    /// Comment associated with the function, if exists.
    pub comment: Option<String>,

    pub kind: FunctionKind,

    pub parameters: Vec<CParameter>,

    pub ret: Return,
}

#[derive(Clone, Debug)]
pub enum Return {
    Value(Option<CParameter>),

    /// If the return is a LVGL result
    ResultError(CParameter),
}
