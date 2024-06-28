use crate::error::*;
use crate::Dtype;

type Result<T, E = ElucidatorError> = std::result::Result<T, E>;

/// The Representable trait must be implemented for any Rust type that can be represented in The
/// Standard. This enables the elucidator library to handle dynamic typing and representations of
/// arbitrary metadata while preserving type safety. The table below indicates which types can
/// safely be converted. Columns indicate the source type, rows indicate the target type, and "x"
/// indicates that the conversion can be performed.
///
/// |        | string | u8 | u16 | u32 | u64 | i16 | i32 | i64 | f32 | f64 |
/// |--------|--------|----|-----|-----|-----|-----|-----|-----|-----|-----|
/// | string | x      |    |     |     |     |     |     |     |     |     |
/// | u8     |        | x  |     |     |     |     |     |     |     |     |
/// | u16    |        | x  | x   |     |     |     |     |     |     |     |
/// | u32    |        | x  | x   | x   |     |     |     |     |     |     |
/// | u64    |        | x  | x   | x   | x   |     |     |     |     |     |
/// | i16    |        | x  |     |     |     | x   |     |     |     |     |
/// | i32    |        | x  | x   |     |     | x   | x   |     |     |     |
/// | i64    |        | x  | x   | x   |     | x   | x   | x   |     |     |
/// | f32    |        | x  | x   |     |     | x   |     |     |     |     |
/// | f64    |        | x  | x   | x   |     | x   | x   |     | x   | x   |
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
/// use elucidator::Dtype;
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
/// use elucidator::Dtype;
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
    fn is_signed(&self) -> bool;
    /// Determine whether this type is an integer
    fn is_integer(&self) -> bool;
    /// Determine whether this type is floating-point
    fn is_floating(&self) -> bool;
    /// Produce an equivalent buffer of bytes
    fn as_buffer(&self) -> Vec<u8>;
    /// Attempt to convert this type into a u64
    fn as_u64(&self) -> Result<u64>;
    /// Attempt to convert this type into an i64
    fn as_i64(&self) -> Result<i64>;
    /// Attempt to convert this type into an f64
    fn as_f64(&self) -> Result<f64>;
}

// TODO: do this as a macro

impl Representable for u8 {
    fn is_numeric(&self) -> bool { true }
    fn is_array(&self) -> bool { false }
    fn get_dtype(&self) -> Dtype { Dtype::Byte }
    fn is_signed(&self) -> bool { false }
    fn is_integer(&self) -> bool { true }
    fn is_floating(&self) -> bool { false }
    fn as_buffer(&self) -> Vec<u8> { self.to_le_bytes().iter().map(|x| *x).collect() }
    fn as_u64(&self) -> Result<u64> { Ok(*self as u64) }
    fn as_i64(&self) -> Result<i64> { Ok(*self as i64) }
    fn as_f64(&self) -> Result<f64> { Ok(f64::from(*self)) }
}

impl Representable for u16 {
    fn is_numeric(&self) -> bool { true }
    fn is_array(&self) -> bool { false }
    fn get_dtype(&self) -> Dtype { Dtype::UnsignedInteger16 }
    fn is_signed(&self) -> bool { false }
    fn is_integer(&self) -> bool { true }
    fn is_floating(&self) -> bool { false }
    fn as_buffer(&self) -> Vec<u8> { self.to_le_bytes().iter().map(|x| *x).collect() }
    fn as_u64(&self) -> Result<u64> { Ok(*self as u64) }
    fn as_i64(&self) -> Result<i64> { Ok(*self as i64) }
    fn as_f64(&self) -> Result<f64> { Ok(f64::from(*self)) }
}

impl Representable for u32 {
    fn is_numeric(&self) -> bool { true }
    fn is_array(&self) -> bool { false }
    fn get_dtype(&self) -> Dtype { Dtype::UnsignedInteger32 }
    fn is_signed(&self) -> bool { false }
    fn is_integer(&self) -> bool { true }
    fn is_floating(&self) -> bool { false }
    fn as_buffer(&self) -> Vec<u8> { self.to_le_bytes().iter().map(|x| *x).collect() }
    fn as_u64(&self) -> Result<u64> { Ok(*self as u64) }
    fn as_i64(&self) -> Result<i64> { Ok(*self as i64) }
    fn as_f64(&self) -> Result<f64> { Ok(f64::from(*self)) }
}

impl Representable for u64 {
    fn is_numeric(&self) -> bool { true }
    fn is_array(&self) -> bool { false }
    fn get_dtype(&self) -> Dtype { Dtype::UnsignedInteger64 }
    fn is_signed(&self) -> bool { false }
    fn is_integer(&self) -> bool { true }
    fn is_floating(&self) -> bool { false }
    fn as_buffer(&self) -> Vec<u8> { self.to_le_bytes().iter().map(|x| *x).collect() }
    fn as_u64(&self) -> Result<u64> { Ok(*self) }
    fn as_i64(&self) -> Result<i64> { ElucidatorError::new_narrowing("u64", "i64") }
    fn as_f64(&self) -> Result<f64> { ElucidatorError::new_narrowing("u64", "f64") }
}

impl Representable for i8 {
    fn is_numeric(&self) -> bool { true }
    fn is_array(&self) -> bool { false }
    fn get_dtype(&self) -> Dtype { Dtype::SignedInteger8 }
    fn is_signed(&self) -> bool { true }
    fn is_integer(&self) -> bool { true }
    fn is_floating(&self) -> bool { false }
    fn as_buffer(&self) -> Vec<u8> { self.to_le_bytes().iter().map(|x| *x).collect() }
    fn as_u64(&self) -> Result<u64> { ElucidatorError::new_narrowing("i8", "u64") }
    fn as_i64(&self) -> Result<i64> { Ok(*self as i64) }
    fn as_f64(&self) -> Result<f64> { Ok(f64::from(*self)) }
}

impl Representable for i16 {
    fn is_numeric(&self) -> bool { true }
    fn is_array(&self) -> bool { false }
    fn get_dtype(&self) -> Dtype { Dtype::SignedInteger16 }
    fn is_signed(&self) -> bool { true }
    fn is_integer(&self) -> bool { true }
    fn is_floating(&self) -> bool { false }
    fn as_buffer(&self) -> Vec<u8> { self.to_le_bytes().iter().map(|x| *x).collect() }
    fn as_u64(&self) -> Result<u64> { ElucidatorError::new_narrowing("i16", "u64") }
    fn as_i64(&self) -> Result<i64> { Ok(*self as i64) }
    fn as_f64(&self) -> Result<f64> { Ok(f64::from(*self)) }
}

impl Representable for i32 {
    fn is_numeric(&self) -> bool { true }
    fn is_array(&self) -> bool { false }
    fn get_dtype(&self) -> Dtype { Dtype::SignedInteger32 }
    fn is_signed(&self) -> bool { true }
    fn is_integer(&self) -> bool { true }
    fn is_floating(&self) -> bool { false }
    fn as_buffer(&self) -> Vec<u8> { self.to_le_bytes().iter().map(|x| *x).collect() }
    fn as_u64(&self) -> Result<u64> { ElucidatorError::new_narrowing("i32", "u64") }
    fn as_i64(&self) -> Result<i64> { Ok(*self as i64) }
    fn as_f64(&self) -> Result<f64> { Ok(f64::from(*self)) }
}

impl Representable for i64 {
    fn is_numeric(&self) -> bool { true }
    fn is_array(&self) -> bool { false }
    fn get_dtype(&self) -> Dtype { Dtype::SignedInteger64 }
    fn is_signed(&self) -> bool { true }
    fn is_integer(&self) -> bool { true }
    fn is_floating(&self) -> bool { false }
    fn as_buffer(&self) -> Vec<u8> { self.to_le_bytes().iter().map(|x| *x).collect() }
    fn as_u64(&self) -> Result<u64> { ElucidatorError::new_narrowing("i64", "u64") }
    fn as_i64(&self) -> Result<i64> { Ok(*self) }
    fn as_f64(&self) -> Result<f64> { ElucidatorError::new_narrowing("i64", "f64")}
}

impl Representable for f32 {
    fn is_numeric(&self) -> bool { true }
    fn is_array(&self) -> bool { false }
    fn get_dtype(&self) -> Dtype { Dtype::Float32 }
    fn is_signed(&self) -> bool { true }
    fn is_integer(&self) -> bool { false }
    fn is_floating(&self) -> bool { true }
    fn as_buffer(&self) -> Vec<u8> { self.to_le_bytes().iter().map(|x| *x).collect() }
    fn as_u64(&self) -> Result<u64> { ElucidatorError::new_narrowing("f32", "u64") }
    fn as_i64(&self) -> Result<i64> { ElucidatorError::new_narrowing("f32", "i64") }
    fn as_f64(&self) -> Result<f64> { Ok(*self as f64) }
}

impl Representable for f64 {
    fn is_numeric(&self) -> bool { true }
    fn is_array(&self) -> bool { false }
    fn get_dtype(&self) -> Dtype { Dtype::Float64 }
    fn is_signed(&self) -> bool { true }
    fn is_integer(&self) -> bool { false }
    fn is_floating(&self) -> bool { true }
    fn as_buffer(&self) -> Vec<u8> { self.to_le_bytes().iter().map(|x| *x).collect() }
    fn as_u64(&self) -> Result<u64> { ElucidatorError::new_narrowing("f64", "u64") }
    fn as_i64(&self) -> Result<i64> { ElucidatorError::new_narrowing("f64", "i64") }
    fn as_f64(&self) -> Result<f64> { Ok(*self) }
}

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
            .as_bytes()
            .iter()
            .map(|x| *x)
            .collect();
        let buffer_len = contents_buffer.len() as u64;
        let mut buffer_indicating_size: Vec<u8> = buffer_len
            .to_le_bytes()
            .iter()
            .map(|x| *x)
            .collect();
        let mut final_buffer = Vec::with_capacity(
            buffer_indicating_size.len() + contents_buffer.len()
        );
        final_buffer.append(&mut buffer_indicating_size);
        final_buffer.append(&mut contents_buffer);
        final_buffer
    }
    fn as_u64(&self) -> Result<u64> { ElucidatorError::new_conversion("string", "u64") }
    fn as_i64(&self) -> Result<i64> { ElucidatorError::new_conversion("string", "i64") }
    fn as_f64(&self) -> Result<f64> { ElucidatorError::new_conversion("string", "f64") }
}

#[cfg(test)]
mod tests {
    use super::*;

    // u8 (byte) -> X
    #[test]
    fn u8_to_u64() {
        let from = u8::MAX / 2;
        let to = from.as_u64();
        assert_eq!(to, Ok(from as u64));
    }
    #[test]
    fn u8_to_i64() {
        let from = u8::MAX / 2;
        let to = from.as_i64();
        assert_eq!(to, Ok(from as i64)); 
    }
    #[test]
    fn u8_to_f64() {
        let from = u8::MAX / 2;
        let to = from.as_f64();
        assert_eq!(to, Ok(from as f64)); 
    }

    // u16 -> X
    #[test]
    fn u16_to_u64() {
        let from = u16::MAX / 2;
        let to = from.as_u64();
        assert_eq!(to, Ok(from as u64));
    }
    #[test]
    fn u16_to_i64() {
        let from = u16::MAX / 2;
        let to = from.as_i64();
        assert_eq!(to, Ok(from as i64)); 
    }
    #[test]
    fn u16_to_f64() {
        let from = u16::MAX / 2;
        let to = from.as_f64();
        assert_eq!(to, Ok(from as f64)); 
    }

    // u32 -> X
    #[test]
    fn u32_to_u64() {
        let from = u32::MAX / 2;
        let to = from.as_u64();
        assert_eq!(to, Ok(from as u64));
    }
    #[test]
    fn u32_to_i64() {
        let from = u32::MAX / 2;
        let to = from.as_i64();
        assert_eq!(to, Ok(from as i64)); 
    }
    #[test]
    fn u32_to_f64() {
        let from = u32::MAX / 2;
        let to = from.as_f64();
        assert_eq!(to, Ok(from as f64)); 
    } 

    // u64 -> X
    #[test]
    fn u64_to_u64() {
        let from = u64::MAX / 2;
        let to = from.as_u64();
        assert_eq!(to, Ok(from as u64));
    }
    #[test]
    fn u64_to_i64() {
        let num = u64::MAX;
        let to = num.as_i64();
        assert_eq!(to, ElucidatorError::new_narrowing("u64", "i64"));
    }
    #[test]
    fn u64_to_f64() {
        let from = u64::MAX / 2;
        let to = from.as_f64();
        assert_eq!(to, ElucidatorError::new_narrowing("u64", "f64")); 
    }

    // i8 -> X
    #[test]
    fn i8_to_u64() {
        let from: i8 = -100;
        let to = from.as_u64();
        assert_eq!(to, ElucidatorError::new_narrowing("i8", "u64"));
    }
    #[test]
    fn i8_to_i64() {
        let from: i8 = -100;
        let to = from.as_i64();
        assert_eq!(to, Ok(from as i64)); 
    }
    #[test]
    fn i8_to_f64() {
        let from: i8 = -100;
        let to = from.as_f64();
        assert_eq!(to, Ok(from as f64)); 
    } 

    // i16 -> X
    #[test]
    fn i16_to_u64() {
        let from: i16 = -100;
        let to = from.as_u64();
        assert_eq!(to, ElucidatorError::new_narrowing("i16", "u64"));
    }
    #[test]
    fn i16_to_i64() {
        let from: i16 = -100;
        let to = from.as_i64();
        assert_eq!(to, Ok(from as i64)); 
    }
    #[test]
    fn i16_to_f64() {
        let from: i16 = -100;
        let to = from.as_f64();
        assert_eq!(to, Ok(from as f64)); 
    }

    // i32 -> X
    #[test]
    fn i32_to_u64() {
        let from: i32 = -100;
        let to = from.as_u64();
        assert_eq!(to, ElucidatorError::new_narrowing("i32", "u64"));
    }
    #[test]
    fn i32_to_i64() {
        let from: i32 = -100;
        let to = from.as_i64();
        assert_eq!(to, Ok(from as i64)); 
    }
    #[test]
    fn i32_to_f64() {
        let from: i32 = -100;
        let to = from.as_f64();
        assert_eq!(to, Ok(from as f64)); 
    }

    // i64 -> X
    #[test]
    fn i64_to_u64() {
        let from: i64 = -100;
        let to = from.as_u64();
        assert_eq!(to, ElucidatorError::new_narrowing("i64", "u64"));
    }
    #[test]
    fn i64_to_i64() {
        let from: i64 = -100;
        let to = from.as_i64();
        assert_eq!(to, Ok(from as i64)); 
    }
    #[test]
    fn i64_to_f64() {
        let from: i64 = -100;
        let to = from.as_f64();
        assert_eq!(to, ElucidatorError::new_narrowing("i64", "f64")); 
    } 

    // f32 -> X
    #[test]
    fn f32_to_u64() {
        let from: f32 = 0.0;
        let to = from.as_u64();
        assert_eq!(to, ElucidatorError::new_narrowing("f32", "u64"));
    }
    #[test]
    fn f32_to_i64() {
        let from: f32 = 0.0;
        let to = from.as_i64();
        assert_eq!(to, ElucidatorError::new_narrowing("f32", "i64"))
    }
    #[test]
    fn f32_to_f64() {
        let from: f32 = 0.0;
        let to = from.as_f64();
        assert_eq!(to, Ok(from as f64)); 
    } 

    // f64 -> X
    #[test]
    fn f64_to_u64() {
        let from: f64 = 0.0;
        let to = from.as_u64();
        assert_eq!(to, ElucidatorError::new_narrowing("f64", "u64"));
    }
    #[test]
    fn f64_to_i64() {
        let from: f64 = 0.0;
        let to = from.as_i64();
        assert_eq!(to, ElucidatorError::new_narrowing("f64", "i64"))
    }
    #[test]
    fn f64_to_f64() {
        let from: f64 = 0.0;
        let to = from.as_f64();
        assert_eq!(to, Ok(from as f64)); 
    } 
    
    // string -> X
    #[test]
    fn string_to_u64() {
        let s = String::from("sunny day");
        let to = s.as_u64();
        assert_eq!(to, ElucidatorError::new_conversion("string", "u64"));
    }
    #[test]
    fn string_to_i64() {
        let s = String::from("sunny day");
        let to = s.as_i64();
        assert_eq!(to, ElucidatorError::new_conversion("string", "i64"));
    }    #[test]
    fn string_to_f64() {
        let s = String::from("sunny day");
        let to = s.as_f64();
        assert_eq!(to, ElucidatorError::new_conversion("string", "f64"));
    }
}
