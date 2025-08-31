use std::any::type_name;

/// Returns the full type name (with module path) of the given value.
///
/// The API is generic and does not allocate: it returns a `&'static str`.
///
/// # Examples
///
/// ```
/// use stdt::utils::type_of::*; 
///
/// let num = 42i32;
/// let text = "hello";
/// let vec = vec![1u8, 2, 3];
///
/// assert!(type_of(&num).ends_with("i32"));
/// assert_eq!(type_of(&text), "&str");
/// assert!(type_of(&vec).contains("Vec<u8>"));
/// ```

pub fn type_of<T>(_: &T) -> &'static str {
    type_name::<T>()
}

/// Returns the short type name (last segment only, without module path).
///
/// Example:
/// - `alloc::vec::Vec<i32>` → `Vec<i32>`
/// - `&str` → `&str`
///
/// # Examples
///
/// ```
/// use stdt::utils::type_of::*;
///
/// let num = 10u32;
/// let text = "hi";
///
/// assert_eq!(type_of_short(&num), "u32");
/// assert_eq!(type_of_short(&text), "&str");
/// ```

pub fn type_of_short<T>(value: &T) -> String {
    type_of(value)
        .rsplit("::")
        .next()
        .unwrap_or(type_of(value))
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::{type_of, type_of_short};

    #[test]
    fn primitive_types() {
        let int_val = 123i32;
        let float_val = 3.14f64;
        let bool_val = true;

        assert!(type_of(&int_val).ends_with("i32"));
        assert_eq!(type_of_short(&int_val), "i32");

        assert!(type_of(&float_val).ends_with("f64"));
        assert_eq!(type_of_short(&float_val), "f64");

        assert!(type_of(&bool_val).ends_with("bool"));
        assert_eq!(type_of_short(&bool_val), "bool");
    }

    #[test]
    fn references_and_slices() {
        let str_val: &str = "hello";
        let byte_slice: &[u8] = &[1, 2, 3];

        assert_eq!(type_of(&str_val), "&str");
        assert_eq!(type_of_short(&str_val), "&str");

        assert_eq!(type_of(&byte_slice), "&[u8]");
        assert_eq!(type_of_short(&byte_slice), "&[u8]");
    }

    #[test]
    fn std_and_alloc_containers() {
        let vec_val = vec![1u8, 2, 3];
        let tuple_val = (1i32, false);

        let t_vec = type_of(&vec_val);
        assert!(t_vec.contains("Vec<u8>"));
        assert_eq!(type_of_short(&vec_val), "Vec<u8>");

        let t_tuple = type_of(&tuple_val);
        assert!(t_tuple.contains("(i32, bool)"));
        assert_eq!(type_of_short(&tuple_val), "(i32, bool)");
    }

    #[test]
    fn user_defined_types() {
        mod inner {
            pub struct Foo;
            pub enum Bar { A }
        }

        let foo_val = inner::Foo;
        let bar_val = inner::Bar::A;

        let t_foo = type_of(&foo_val);
        let t_bar = type_of(&bar_val);

        assert!(t_foo.ends_with("inner::Foo"));
        assert_eq!(type_of_short(&foo_val), "Foo");

        assert!(t_bar.ends_with("inner::Bar"));
        assert_eq!(type_of_short(&bar_val), "Bar");
    }

    #[test]
    fn generics() {
        struct Wrapper<T>(T);

        let wrapped = Wrapper(10u32);
        let t_wrapped = type_of(&wrapped);

        assert!(t_wrapped.ends_with("Wrapper<u32>"));
        assert_eq!(type_of_short(&wrapped), "Wrapper<u32>");
    }
}
