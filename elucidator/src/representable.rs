use crate::error::*;
use crate::member::Dtype;
use elucidator_macros::representable_primitive_impl;

type Result<T, E = ElucidatorError> = std::result::Result<T, E>;

/// The Representable trait must be implemented for any Rust type that can be represented in The
/// Standard. This enables the elucidator library to handle dynamic typing and representations of
/// arbitrary metadata while preserving type safety. The table below indicates which types can
/// safely be converted. Columns indicate the source type, rows indicate the target type, and "x"
/// indicates that the conversion can be performed.
///
/// |        | string | u8 | u16 | u32 | u64 | i8  | i16 | i32 | i64 | f32 | f64 |
/// |--------|--------|----|-----|-----|-----|-----|-----|-----|-----|-----|-----|
/// | string | x      |    |     |     |     |     |     |     |     |     |     |
/// | u8     |        | x  |     |     |     |     |     |     |     |     |     |
/// | u16    |        | x  | x   |     |     |     |     |     |     |     |     |
/// | u32    |        | x  | x   | x   |     |     |     |     |     |     |     |
/// | u64    |        | x  | x   | x   | x   |     |     |     |     |     |     |
/// | i8     |        |    |     |     |     | x   |     |     |     |     |     |
/// | i16    |        | x  |     |     |     | x   | x   |     |     |     |     |
/// | i32    |        | x  | x   |     |     | x   | x   | x   |     |     |     |
/// | i64    |        | x  | x   | x   |     | x   | x   | x   | x   |     |     |
/// | f32    |        | x  | x   |     |     | x   | x   |     |     | x   |     |
/// | f64    |        | x  | x   | x   |     | x   | x   | x   |     | x   | x   |
/// 
/// # Examples
/// 
/// All examples presume you insert the following use statement:
/// ```
/// use elucidator::Representable;
/// ```
///
/// Many examples also presume that you're receiving a `Box<dyn Representable>`, `datum`.
/// You can make one like this:
/// ```
/// # use elucidator::Representable;
/// let datum: Box<dyn Representable> = Box::new(0);
/// ```
///
/// You can introspect about the qualities of a value extracted at runtime
///
/// ```
/// # use elucidator::Representable;
/// # let datum: Box<dyn Representable> = Box::new(5);
/// if datum.is_numeric() {
///     // Is it an integer or a float?
///     if datum.is_integer() {
///         // Great!
///     } else {
///         // Uh-oh, we were looking for an integer, panic!
///         panic!("We needed an integer!");
///     }
/// } else {
///     // Uh-oh, we wanted something numerical!
///     panic!("Value is non-numeric!")
/// }
/// ```
/// or you can check the `Dtype` and perform a cast, knowing it should succeed
/// ```
/// # use elucidator::Representable;
/// # let datum: Box<dyn Representable> = Box::new(0 as u8);
/// use elucidator::member::Dtype;
///
/// let datum: Box<dyn Representable> = Box::new(0.0 as f64);
/// let extracted_val = match datum.get_dtype() {
///     Dtype::Float64 => {
///         datum.as_f64().unwrap()
///     },
///     _ => { panic!("Expected a Float64!"); },
/// };
/// # assert_eq!(extracted_val, 0.0);
/// ```
/// or just attempt to perform a cast without knowing the type, handling error conditions
/// accordingly:
/// ```
/// # use elucidator::Representable;
/// # let datum: Box<dyn Representable> = Box::new(0 as u8);
/// let mut counts = 0;
///
/// match datum.as_i64() {
///     Ok(val) => { counts += val; },
///     Err(e) => {
///         eprintln!("{e}");
///     },
/// }
/// # assert_eq!(datum.as_i64(), Ok(0 as i64));
/// ```
/// For collections of unknown types, you can filter on the Dtype and convert them:
/// ```
/// # use elucidator::Representable;
/// # let unknown_types: Vec<Box<dyn Representable>> = vec![
/// #   Box::new(0 as u8),
/// #   Box::new("cat".to_string()),
/// #   Box::new(4.5 as f64),
/// #   Box::new(5.4 as f64),
/// # ];
/// use elucidator::member::Dtype;
///
/// let floats: Vec<f64> = unknown_types.iter()
///     .filter(|x| x.get_dtype() == Dtype::Float64)
///     .map(|x| x.as_f64().unwrap())
///     .collect();
/// # assert_eq!(floats, vec![4.5, 5.4]);
/// ```
/// You can also convert into a buffer to prepare for storage:
/// ```
/// # use elucidator::Representable;
/// # let datum: Box<dyn Representable> = Box::new(0 as u8);
/// let datum_as_buffer = datum.as_buffer();
/// ```

pub trait Representable {
    /// Determine whether this type contains numeric values
    fn is_numeric(&self) -> bool;
    /// Determine if this type is an array
    fn is_array(&self) -> bool;
    /// Return the Dtype of this object
    fn get_dtype(&self) -> Dtype;
    /// Determine whether this type is signed
    fn is_signed(&self) -> bool;
    /// Determine whether this type is an integer
    fn is_integer(&self) -> bool;
    /// Determine whether this type is floating-point
    fn is_floating(&self) -> bool;
    /// Produce an equivalent buffer of bytes
    fn as_buffer(&self) -> Vec<u8>;
    /// Attempt to convert this type into a u8
    fn as_u8(&self) -> Result<u8, ElucidatorError>;
    /// Attempt to convert this type into a u16
    fn as_u16(&self) -> Result<u16, ElucidatorError>;
    /// Attempt to convert this type into a u32
    fn as_u32(&self) -> Result<u32, ElucidatorError>;
    /// Attempt to convert this type into a u64
    fn as_u64(&self) -> Result<u64, ElucidatorError>;
    /// Attempt to convert this type into a i8
    fn as_i8(&self) -> Result<i8, ElucidatorError>;
    /// Attempt to convert this type into a i16
    fn as_i16(&self) -> Result<i16, ElucidatorError>;
    /// Attempt to convert this type into a i32
    fn as_i32(&self) -> Result<i32, ElucidatorError>;
    /// Attempt to convert this type into a i64
    fn as_i64(&self) -> Result<i64, ElucidatorError>;
    /// Attempt to convert this type into a f32
    fn as_f32(&self) -> Result<f32, ElucidatorError>;
    /// Attempt to convert this type into a f64
    fn as_f64(&self) -> Result<f64, ElucidatorError>;
    fn as_string(&self) -> Result<String, ElucidatorError>;
}

representable_primitive_impl!(std::primitive::u8);
representable_primitive_impl!(std::primitive::u16);
representable_primitive_impl!(std::primitive::u32);
representable_primitive_impl!(std::primitive::u64);
representable_primitive_impl!(std::primitive::i8);
representable_primitive_impl!(std::primitive::i16);
representable_primitive_impl!(std::primitive::i32);
representable_primitive_impl!(std::primitive::i64);
representable_primitive_impl!(std::primitive::f32);
representable_primitive_impl!(std::primitive::f64);

impl Representable for String {
    fn is_numeric(&self) -> bool { false }
    fn is_array(&self) -> bool { false }
    fn get_dtype(&self) -> Dtype { Dtype::Str }
    fn is_signed(&self) -> bool { false }
    fn is_integer(&self) -> bool { false }
    fn is_floating(&self) -> bool { false }
    fn as_buffer(&self) -> Vec<u8> {
        // TODO: Determine if we need to enforce ASCII
        let mut contents_buffer: Vec<u8> = self
            .as_bytes().to_vec();
        let buffer_len = contents_buffer.len() as u64;
        let mut buffer_indicating_size: Vec<u8> = buffer_len
            .to_le_bytes().to_vec();
        let mut final_buffer = Vec::with_capacity(
            buffer_indicating_size.len() + contents_buffer.len()
        );
        final_buffer.append(&mut buffer_indicating_size);
        final_buffer.append(&mut contents_buffer);
        final_buffer
    }
    fn as_u8(&self) -> Result<u8, ElucidatorError> {
        ElucidatorError::new_conversion("string", "u8")
    }
    fn as_u16(&self) -> Result<u16, ElucidatorError> {
        ElucidatorError::new_conversion("string", "u16")
    }
    fn as_u32(&self) -> Result<u32, ElucidatorError> {
        ElucidatorError::new_conversion("string", "u32")
    }
    fn as_u64(&self) -> Result<u64, ElucidatorError> {
        ElucidatorError::new_conversion("string", "u64")
    }
    fn as_i8(&self) -> Result<i8, ElucidatorError> {
        ElucidatorError::new_conversion("string", "i8")
    }
    fn as_i16(&self) -> Result<i16, ElucidatorError> {
        ElucidatorError::new_conversion("string", "i16")
    }
    fn as_i32(&self) -> Result<i32, ElucidatorError> {
        ElucidatorError::new_conversion("string", "i32")
    }
    fn as_i64(&self) -> Result<i64, ElucidatorError> {
        ElucidatorError::new_conversion("string", "i64")
    }
    fn as_f32(&self) -> Result<f32, ElucidatorError> {
        ElucidatorError::new_conversion("string", "f32")
    }
    fn as_f64(&self) -> Result<f64, ElucidatorError> {
        ElucidatorError::new_conversion("string", "f64")
    }
    fn as_string(&self) -> Result<String, ElucidatorError> { Ok(self.clone()) }
}

#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! conversion_test {
    ($source_type:ty, $conversion_fn:ident, $fn_name:ident, $expected:expr) => {
        #[test]
        fn $fn_name() {
            let source: $source_type = <$source_type>::default();
            let received = source.$conversion_fn();
            assert_eq!(received, $expected);
        }
    }
    }

    conversion_test!(u8, as_u8,     u8_to_u8, Ok(u8::default()));
    conversion_test!(u8, as_u16,    u8_to_u16, Ok(u16::default()));
    conversion_test!(u8, as_u32,    u8_to_u32, Ok(u32::default()));
    conversion_test!(u8, as_u64,    u8_to_u64, Ok(u64::default()));
    conversion_test!(u8, as_i8,     u8_to_i8, ElucidatorError::new_narrowing("u8", "i8"));
    conversion_test!(u8, as_i16,    u8_to_i16, Ok(i16::default()));
    conversion_test!(u8, as_i32,    u8_to_i32, Ok(i32::default()));
    conversion_test!(u8, as_i64,    u8_to_i64, Ok(i64::default()));
    conversion_test!(u8, as_f32,    u8_to_f32, Ok(f32::default()));
    conversion_test!(u8, as_f64,    u8_to_f64, Ok(f64::default()));
    conversion_test!(u8, as_string, u8_as_string, ElucidatorError::new_conversion("u8", "string"));

    conversion_test!(u16, as_u8,     u16_to_u8, ElucidatorError::new_narrowing("u16", "u8"));
    conversion_test!(u16, as_u16,    u16_to_u16, Ok(u16::default()));
    conversion_test!(u16, as_u32,    u16_to_u32, Ok(u32::default()));
    conversion_test!(u16, as_u64,    u16_to_u64, Ok(u64::default()));
    conversion_test!(u16, as_i8,     u16_to_i8, ElucidatorError::new_narrowing("u16", "i8"));
    conversion_test!(u16, as_i16,    u16_to_i16, ElucidatorError::new_narrowing("u16", "i16"));
    conversion_test!(u16, as_i32,    u16_to_i32, Ok(i32::default()));
    conversion_test!(u16, as_i64,    u16_to_i64, Ok(i64::default()));
    conversion_test!(u16, as_f32,    u16_to_f32, Ok(f32::default()));
    conversion_test!(u16, as_f64,    u16_to_f64, Ok(f64::default()));
    conversion_test!(u16, as_string, u16_as_string, ElucidatorError::new_conversion("u16", "string"));

    conversion_test!(u32, as_u8,     u32_to_u8, ElucidatorError::new_narrowing("u32", "u8"));
    conversion_test!(u32, as_u16,    u32_to_u16, ElucidatorError::new_narrowing("u32", "u16"));
    conversion_test!(u32, as_u32,    u32_to_u32, Ok(u32::default()));
    conversion_test!(u32, as_u64,    u32_to_u64, Ok(u64::default()));
    conversion_test!(u32, as_i8,     u32_to_i8, ElucidatorError::new_narrowing("u32", "i8"));
    conversion_test!(u32, as_i16,    u32_to_i16, ElucidatorError::new_narrowing("u32", "i16"));
    conversion_test!(u32, as_i32,    u32_to_i32, ElucidatorError::new_narrowing("u32", "i32"));
    conversion_test!(u32, as_i64,    u32_to_i64, Ok(i64::default()));
    conversion_test!(u32, as_f32,    u32_to_f32, ElucidatorError::new_narrowing("u32", "f32"));
    conversion_test!(u32, as_f64,    u32_to_f64, Ok(f64::default()));
    conversion_test!(u32, as_string, u32_as_string, ElucidatorError::new_conversion("u32", "string"));

    conversion_test!(u64, as_u8,     u64_to_u8, ElucidatorError::new_narrowing("u64", "u8"));
    conversion_test!(u64, as_u16,    u64_to_u16, ElucidatorError::new_narrowing("u64", "u16"));
    conversion_test!(u64, as_u32,    u64_to_u32, ElucidatorError::new_narrowing("u64", "u32"));
    conversion_test!(u64, as_u64,    u64_to_u64, Ok(u64::default()));
    conversion_test!(u64, as_i8,     u64_to_i8, ElucidatorError::new_narrowing("u64", "i8"));
    conversion_test!(u64, as_i16,    u64_to_i16, ElucidatorError::new_narrowing("u64", "i16"));
    conversion_test!(u64, as_i32,    u64_to_i32, ElucidatorError::new_narrowing("u64", "i32"));
    conversion_test!(u64, as_i64,    u64_to_i64, ElucidatorError::new_narrowing("u64", "i64"));
    conversion_test!(u64, as_f32,    u64_to_f32, ElucidatorError::new_narrowing("u64", "f32"));
    conversion_test!(u64, as_f64,    u64_to_f64, ElucidatorError::new_narrowing("u64", "f64"));
    conversion_test!(u64, as_string, u64_as_string, ElucidatorError::new_conversion("u64", "string"));

    conversion_test!(i8, as_u8,     i8_to_u8, ElucidatorError::new_narrowing("i8", "u8"));
    conversion_test!(i8, as_u16,    i8_to_u16, ElucidatorError::new_narrowing("i8", "u16"));
    conversion_test!(i8, as_u32,    i8_to_u32, ElucidatorError::new_narrowing("i8", "u32"));
    conversion_test!(i8, as_u64,    i8_to_u64, ElucidatorError::new_narrowing("i8", "u64"));
    conversion_test!(i8, as_i8,     i8_to_i8, Ok(i8::default()));
    conversion_test!(i8, as_i16,    i8_to_i16, Ok(i16::default()));
    conversion_test!(i8, as_i32,    i8_to_i32, Ok(i32::default()));
    conversion_test!(i8, as_i64,    i8_to_i64, Ok(i64::default()));
    conversion_test!(i8, as_f32,    i8_to_f32, Ok(f32::default()));
    conversion_test!(i8, as_f64,    i8_to_f64, Ok(f64::default()));
    conversion_test!(i8, as_string, i8_as_string, ElucidatorError::new_conversion("i8", "string"));

    conversion_test!(i16, as_u8,     i16_to_u8, ElucidatorError::new_narrowing("i16", "u8"));
    conversion_test!(i16, as_u16,    i16_to_u16, ElucidatorError::new_narrowing("i16", "u16"));
    conversion_test!(i16, as_u32,    i16_to_u32, ElucidatorError::new_narrowing("i16", "u32"));
    conversion_test!(i16, as_u64,    i16_to_u64, ElucidatorError::new_narrowing("i16", "u64"));
    conversion_test!(i16, as_i8,     i16_to_i8, ElucidatorError::new_narrowing("i16", "i8"));
    conversion_test!(i16, as_i16,    i16_to_i16, Ok(i16::default()));
    conversion_test!(i16, as_i32,    i16_to_i32, Ok(i32::default()));
    conversion_test!(i16, as_i64,    i16_to_i64, Ok(i64::default()));
    conversion_test!(i16, as_f32,    i16_to_f32, Ok(f32::default()));
    conversion_test!(i16, as_f64,    i16_to_f64, Ok(f64::default()));
    conversion_test!(i16, as_string, i16_as_string, ElucidatorError::new_conversion("i16", "string"));

    conversion_test!(i32, as_u8,     i32_to_u8, ElucidatorError::new_narrowing("i32", "u8"));
    conversion_test!(i32, as_u16,    i32_to_u16, ElucidatorError::new_narrowing("i32", "u16"));
    conversion_test!(i32, as_u32,    i32_to_u32, ElucidatorError::new_narrowing("i32", "u32"));
    conversion_test!(i32, as_u64,    i32_to_u64, ElucidatorError::new_narrowing("i32", "u64"));
    conversion_test!(i32, as_i8,     i32_to_i8, ElucidatorError::new_narrowing("i32", "i8"));
    conversion_test!(i32, as_i16,    i32_to_i16, ElucidatorError::new_narrowing("i32", "i16"));
    conversion_test!(i32, as_i32,    i32_to_i32, Ok(i32::default()));
    conversion_test!(i32, as_i64,    i32_to_i64, Ok(i64::default()));
    conversion_test!(i32, as_f32,    i32_to_f32, ElucidatorError::new_narrowing("i32", "f32"));
    conversion_test!(i32, as_f64,    i32_to_f64, Ok(f64::default()));
    conversion_test!(i32, as_string, i32_as_string, ElucidatorError::new_conversion("i32", "string"));

    conversion_test!(i64, as_u8,     i64_to_u8, ElucidatorError::new_narrowing("i64", "u8"));
    conversion_test!(i64, as_u16,    i64_to_u16, ElucidatorError::new_narrowing("i64", "u16"));
    conversion_test!(i64, as_u32,    i64_to_u32, ElucidatorError::new_narrowing("i64", "u32"));
    conversion_test!(i64, as_u64,    i64_to_u64, ElucidatorError::new_narrowing("i64", "u64"));
    conversion_test!(i64, as_i8,     i64_to_i8, ElucidatorError::new_narrowing("i64", "i8"));
    conversion_test!(i64, as_i16,    i64_to_i16, ElucidatorError::new_narrowing("i64", "i16"));
    conversion_test!(i64, as_i32,    i64_to_i32, ElucidatorError::new_narrowing("i64", "i32"));
    conversion_test!(i64, as_i64,    i64_to_i64, Ok(i64::default()));
    conversion_test!(i64, as_f32,    i64_to_f32, ElucidatorError::new_narrowing("i64", "f32"));
    conversion_test!(i64, as_f64,    i64_to_f64, ElucidatorError::new_narrowing("i64", "f64"));
    conversion_test!(i64, as_string, i64_as_string, ElucidatorError::new_conversion("i64", "string"));

    conversion_test!(f32, as_u8,     f32_to_u8, ElucidatorError::new_narrowing("f32", "u8"));
    conversion_test!(f32, as_u16,    f32_to_u16, ElucidatorError::new_narrowing("f32", "u16"));
    conversion_test!(f32, as_u32,    f32_to_u32, ElucidatorError::new_narrowing("f32", "u32"));
    conversion_test!(f32, as_u64,    f32_to_u64, ElucidatorError::new_narrowing("f32", "u64"));
    conversion_test!(f32, as_i8,     f32_to_i8, ElucidatorError::new_narrowing("f32", "i8"));
    conversion_test!(f32, as_i16,    f32_to_i16, ElucidatorError::new_narrowing("f32", "i16"));
    conversion_test!(f32, as_i32,    f32_to_i32, ElucidatorError::new_narrowing("f32", "i32"));
    conversion_test!(f32, as_i64,    f32_to_i64, ElucidatorError::new_narrowing("f32", "i64"));
    conversion_test!(f32, as_f32,    f32_to_f32, Ok(f32::default()));
    conversion_test!(f32, as_f64,    f32_to_f64, Ok(f64::default()));
    conversion_test!(f32, as_string, f32_as_string, ElucidatorError::new_conversion("f32", "string"));

    conversion_test!(f64, as_u8,     f64_to_u8, ElucidatorError::new_narrowing("f64", "u8"));
    conversion_test!(f64, as_u16,    f64_to_u16, ElucidatorError::new_narrowing("f64", "u16"));
    conversion_test!(f64, as_u32,    f64_to_u32, ElucidatorError::new_narrowing("f64", "u32"));
    conversion_test!(f64, as_u64,    f64_to_u64, ElucidatorError::new_narrowing("f64", "u64"));
    conversion_test!(f64, as_i8,     f64_to_i8, ElucidatorError::new_narrowing("f64", "i8"));
    conversion_test!(f64, as_i16,    f64_to_i16, ElucidatorError::new_narrowing("f64", "i16"));
    conversion_test!(f64, as_i32,    f64_to_i32, ElucidatorError::new_narrowing("f64", "i32"));
    conversion_test!(f64, as_i64,    f64_to_i64, ElucidatorError::new_narrowing("f64", "i64"));
    conversion_test!(f64, as_f32,    f64_to_f32, ElucidatorError::new_narrowing("f64", "f32"));
    conversion_test!(f64, as_f64,    f64_to_f64, Ok(f64::default()));
    conversion_test!(f64, as_string, f64_as_string, ElucidatorError::new_conversion("f64", "string"));

    conversion_test!(String, as_u8, string_to_u8, ElucidatorError::new_conversion("string", "u8"));
    conversion_test!(String, as_u16, string_to_u16, ElucidatorError::new_conversion("string", "u16"));
    conversion_test!(String, as_u32, string_to_u32, ElucidatorError::new_conversion("string", "u32"));
    conversion_test!(String, as_u64, string_to_u64, ElucidatorError::new_conversion("string", "u64"));
    conversion_test!(String, as_i8, string_to_i8, ElucidatorError::new_conversion("string", "i8"));
    conversion_test!(String, as_i16, string_to_i16, ElucidatorError::new_conversion("string", "i16"));
    conversion_test!(String, as_i32, string_to_i32, ElucidatorError::new_conversion("string", "i32"));
    conversion_test!(String, as_i64, string_to_i64, ElucidatorError::new_conversion("string", "i64"));
    conversion_test!(String, as_f32, string_to_f32, ElucidatorError::new_conversion("string", "f32"));
    conversion_test!(String, as_f64, string_to_f64, ElucidatorError::new_conversion("string", "f64"));
    conversion_test!(String, as_string, string_to_string, Ok(String::default()));
}
