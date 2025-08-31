/// # type_of
///
/// Small utility to get the type name of a variable at runtime.
///
/// ## Overview
/// - `type_of(&x)` returns the full type name of `x` (with module path),
///   e.g. `alloc::vec::Vec<i32>` or `&str`.
/// - `type_of_short(&x)` returns only the last segment (without module path),
///   e.g. `Vec<i32>` or `&str`.
///
/// ## Limitations
/// - Type aliases are resolved: you will see the underlying type.
/// - The module path (e.g., `alloc::...`, `crate::...`) may vary across versions/environments.

pub mod type_of;
