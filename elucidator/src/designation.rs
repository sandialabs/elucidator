use std::collections::HashMap;
use std::io::{Cursor, Read};

use crate::{
    error::*,
    member::{MemberSpecification, Sizing, Dtype},
    parsing,
    validating,
    value::{DataValue, LeBufferRead},
    representable::Representable,
};

use elucidator_macros::make_dtype_interpreter;

type Result<T, E = ElucidatorError> = std::result::Result<T, E>;

make_dtype_interpreter!(u8);
make_dtype_interpreter!(u16);
make_dtype_interpreter!(u32);
make_dtype_interpreter!(u64);
make_dtype_interpreter!(i8);
make_dtype_interpreter!(i16);
make_dtype_interpreter!(i32);
make_dtype_interpreter!(i64);
make_dtype_interpreter!(f32);
make_dtype_interpreter!(f64);

/// Representation of a Designation's specification.
/// Use to parse a specification for an individual designation.
/// To construct, it is typical to use the `from_text` method.
/// ```
/// use elucidator::designation::DesignationSpecification;
/// # use elucidator::member::{Dtype, MemberSpecification, Sizing};
///
/// let spec = DesignationSpecification::from_text("foo: u32");
///
/// # assert!(spec.is_ok())
/// ```
#[derive(Debug, PartialEq, Clone)]
pub struct DesignationSpecification {
    members: Vec<MemberSpecification>,
}

fn subselect_text(text: &str, start: usize, end: usize) -> (&str, usize) {
    let end = if text.chars().count() <= end {
        text.chars().count() - 1
    } else {
        end
    };
    let (start_byte_pos, _) = text.char_indices().nth(start).unwrap();
    let (end_byte_pos, _) = text.char_indices().nth(end).unwrap();
    let last_comma_pos = text[..start_byte_pos]
        .rfind(',');
    let selection_end = match text[end_byte_pos..].find(',') {
        Some(pos) => pos + end_byte_pos,
        None => text.len(),
    };
    // Relative to last_spec_pos
    let selection_start = match last_comma_pos {
        Some(pos) => {
            let (char_after_last_spec_pos, _) = text[pos..]
                .char_indices()
                .nth(1)
                .unwrap_or((pos, ' '));
            char_after_last_spec_pos + pos
        },
        None => 0,
    };
    let subselection = &text[selection_start..selection_end];
    let offset = text[..selection_start].chars().count();
    (subselection, offset)
}

fn produce_caret_string(
    subselection: &str,
    start: usize,
    end: usize,
    offset: usize
) -> String {
    (offset..(offset + subselection.chars().count()))
        .map(|x| {
            if x >= start && x < end {
                '^'
            } else {
                ' '
            }
        })
        .collect()
}

fn produce_context(text: &str, start: usize, end: usize) -> String {
    let (selection, offset) = subselect_text(text, start, end);
    let caret_string = produce_caret_string(selection, start, end, offset);
    format!("{selection}\n{caret_string}")
}

fn convert_error(error: &InternalError, text: &str) -> ElucidatorError {
    match error {
        InternalError::Parsing{offender, reason} => {
            let column_start = offender.column_start;
            let column_end = offender.column_end;
            ElucidatorError::Specification {
                context: produce_context(text, column_start, column_end), 
                column_start,
                column_end,
                reason: format!("{reason}\n"),
            }
        },
        InternalError::IllegalSpecification{offender, reason} => {
            let column_start = offender.column_start;
            let column_end = offender.column_end;
            ElucidatorError::Specification {
                context: produce_context(text, column_start, column_end),
                column_start,
                column_end,
                reason: format!("{reason}"),
            }
        },
        InternalError::MultipleFailures(errs) => {
            let errors: Vec<ElucidatorError> = errs.iter()
                .map(|e| convert_error(e, text))
                .collect();
            ElucidatorError::merge(&errors)
        }
    }
}

fn get_sizing_from_buff(cursor: &mut Cursor<&[u8]>) -> Result<usize> {
    let size_bytes = 8;
    let mut member_size_buffer: Vec<u8> = Vec::with_capacity(size_bytes);
    get_n_bytes_from_buff(cursor, &mut member_size_buffer, size_bytes)?;
    Ok(u64::from_le_bytes(member_size_buffer[0..8].try_into().unwrap()) as usize)
}

fn get_n_bytes_from_buff(cursor: &mut Cursor<&[u8]>, buffer: &mut Vec<u8>, n: usize) -> Result<()> {
    let start_pos = cursor.position();

    let mut handle = cursor.take(n as u64);
    match handle.read_to_end(buffer) {
        Ok(m) => { 
            if n != m {
                Err(ElucidatorError::BufferSizing { 
                    expected: n, 
                    found: m
                })?
            }
        },
        Err(e) => {
            eprintln!("{e}");
            Err(ElucidatorError::BufferSizing { 
                expected: buffer.len(), 
                found: (cursor.position() - start_pos) as usize
            })?
        }
    };
    cursor.set_position(start_pos + n as u64);
    Ok(())
}

// DON'T USE THIS EXCEPT INSIDE OF INTERPRETING ENUMS
fn get_singleton_from_buf(buf: &[u8], dt: &Dtype) -> Result<DataValue> {
    match dt {
        Dtype::Byte => { 
            Ok(DataValue::Byte(u8::get_one_le(buf)?))
        },
        Dtype::UnsignedInteger16 => {
            Ok(DataValue::UnsignedInteger16(
                u16::get_one_le(buf)?
            ))
        },
        Dtype::UnsignedInteger32 => {
            Ok(DataValue::UnsignedInteger32(
                u32::get_one_le(buf)?
            ))
        },
        Dtype::UnsignedInteger64 => {
            Ok(DataValue::UnsignedInteger64(
                u64::get_one_le(buf)?
            ))
        },
        Dtype::SignedInteger8 => { 
            Ok(DataValue::SignedInteger8(i8::get_one_le(buf)?))
        },
        Dtype::SignedInteger16 => {
            Ok(DataValue::SignedInteger16(
                i16::get_one_le(buf)?
            ))
        },
        Dtype::SignedInteger32 => {
            Ok(DataValue::SignedInteger32(
                i32::get_one_le(buf)?
            ))
        },
        Dtype::SignedInteger64 => {
            Ok(DataValue::SignedInteger64(
                i64::get_one_le(buf)?
            ))
        },
        Dtype::Float32 => {
            Ok(DataValue::Float32(
                f32::get_one_le(buf)?
            ))
        },
        Dtype::Float64 => {
            Ok(DataValue::Float64(
                f64::get_one_le(buf)?
            ))
        },
        Dtype::Str => {
            Ok(DataValue::Str(
                String::get_one_le(buf)?
            ))
        },
    }
}

// DON'T USE THIS EXCEPT INSIDE OF INTERPRETING ENUMS
fn get_array_from_buf(buf: &[u8], dt: &Dtype, items_to_read: usize) -> Result<DataValue> {
    match dt {
        Dtype::Byte => { 
            Ok(DataValue::ByteArray(u8::get_n_le(buf, items_to_read)?))
        },
        Dtype::UnsignedInteger16 => {
            Ok(DataValue::UnsignedInteger16Array(
                u16::get_n_le(buf, items_to_read)?
            ))
        },
        Dtype::UnsignedInteger32 => {
            Ok(DataValue::UnsignedInteger32Array(
                u32::get_n_le(buf, items_to_read)?
            ))
        },
        Dtype::UnsignedInteger64 => {
            Ok(DataValue::UnsignedInteger64Array(
                u64::get_n_le(buf, items_to_read)?
            ))
        },
        Dtype::SignedInteger8 => { 
            Ok(DataValue::SignedInteger8Array(
                    i8::get_n_le(buf, items_to_read)?
            ))
        },
        Dtype::SignedInteger16 => {
            Ok(DataValue::SignedInteger16Array(
                i16::get_n_le(buf, items_to_read)?
            ))
        },
        Dtype::SignedInteger32 => {
            Ok(DataValue::SignedInteger32Array(
                i32::get_n_le(buf, items_to_read)?
            ))
        },
        Dtype::SignedInteger64 => {
            Ok(DataValue::SignedInteger64Array(
                i64::get_n_le(buf, items_to_read)?
            ))
        },
        Dtype::Float32 => {
            Ok(DataValue::Float32Array(
                f32::get_n_le(buf, items_to_read)?
            ))
        },
        Dtype::Float64 => {
            Ok(DataValue::Float64Array(
                f64::get_n_le(buf, items_to_read)?
            ))
        },
        _ => {
            unreachable!("Match statement has exhausted all array values for buffer reading");
        },
    }
}

impl DesignationSpecification {
    pub fn from_text(text: &str) -> Result<Self> {
        let parsed = parsing::get_metadataspec(text);
        let validated = validating::validate_metadataspec(&parsed);
        match validated {
            Ok(members) => Ok(DesignationSpecification{ members }),
            Err(e) => Err(convert_error(&e, text)),
        }
    }

    pub fn interpret(&self, buffer: &[u8]) -> Result<HashMap<&str, Box<dyn Representable>>> {
        let mut map = HashMap::new();
        let mut cursor = Cursor::new(buffer);
        for member in &self.members {
            let items_to_read: usize = match member.sizing {
                Sizing::Singleton => { 1 },
                Sizing::Fixed(n) => { n as usize },
                Sizing::Dynamic => {
                    get_sizing_from_buff(&mut cursor)?
                }
            };

            let value: Box<dyn Representable> = match &member.dtype {
                Dtype::Str => {
                    let n_bytes = get_sizing_from_buff(&mut cursor)?;
                    let mut string_buffer: Vec<u8> = Vec::with_capacity(n_bytes);
                    get_n_bytes_from_buff(&mut cursor, &mut string_buffer, n_bytes)?;
                    match String::from_utf8(string_buffer) {
                        Ok(s) => { Box::new(s) },
                        Err(e) => {
                            Err(ElucidatorError::FromUtf8 { source: e })?
                        }
                    }
                },
                Dtype::Byte => { interpret_u8(&mut cursor, items_to_read, &member.sizing)? },
                Dtype::UnsignedInteger16 => { interpret_u16(&mut cursor, items_to_read, &member.sizing)? },
                Dtype::UnsignedInteger32 => { interpret_u32(&mut cursor, items_to_read, &member.sizing)? },
                Dtype::UnsignedInteger64 => { interpret_u64(&mut cursor, items_to_read, &member.sizing)? },
                Dtype::SignedInteger8 => { interpret_i8(&mut cursor, items_to_read, &member.sizing)? },
                Dtype::SignedInteger16 => { interpret_i16(&mut cursor, items_to_read, &member.sizing)? },
                Dtype::SignedInteger32 => { interpret_i32(&mut cursor, items_to_read, &member.sizing)? },
                Dtype::SignedInteger64 => { interpret_i64(&mut cursor, items_to_read, &member.sizing)? },
                Dtype::Float32 => { interpret_f32(&mut cursor, items_to_read, &member.sizing)? },
                Dtype::Float64 => { interpret_f64(&mut cursor, items_to_read, &member.sizing)? },
            };
            map.insert(member.identifier.as_str(), value);
        }
        Ok(map)
    }

    pub fn interpret_enum(&self, buffer: &[u8]) -> Result<HashMap<&str, DataValue>> {
        let mut map = HashMap::new();
        let mut cursor = Cursor::new(buffer);
        for member in &self.members {
            let value = match member.sizing {
                Sizing::Singleton => {
                    get_singleton_from_buf(buffer, &member.dtype)? 
                },
                Sizing::Fixed(n) => {
                    get_array_from_buf(buffer, &member.dtype, n as usize)?
                },
                Sizing::Dynamic => {
                    let n = get_sizing_from_buff(&mut cursor)?;
                    let buf = &buffer[(cursor.position() as usize)..];
                    get_array_from_buf(buf, &member.dtype, n)?
                }
            };
            map.insert(member.identifier.as_str(), value);
        }
        Ok(map)
    }
}

#[cfg(test)]
mod test {
    use std::collections::HashSet;

    use super::*;
    use crate::{member::{Dtype, Sizing}, test_utils};
    use pretty_assertions::assert_eq;

    type DataMap<'a> = HashMap<&'a str, Box<dyn Representable>>;

    fn make_dyn_box<T: Representable + 'static>(item: T) -> Box<dyn Representable>{
        Box::new(item)
    }

    fn compare_hashmap(left: &DataMap, right: &DataMap) {
        let left_keys: HashSet<&str> = left.keys().copied().collect();
        let right_keys: HashSet<&str> = right.keys().copied().collect();

        pretty_assertions::assert_eq!(left_keys, right_keys);

        for key in left_keys {
            let lvalue= left.get(key).unwrap();
            let rvalue = right.get(key).unwrap();

            pretty_assertions::assert_eq!(lvalue.get_dtype(), rvalue.get_dtype());
            pretty_assertions::assert_eq!(lvalue.is_array(), rvalue.is_array()); 
            
            if lvalue.is_array() {
                match lvalue.get_dtype() {
                    Dtype::Byte => { pretty_assertions::assert_eq!(lvalue.as_vec_u8().unwrap(), rvalue.as_vec_u8().unwrap()); },
                    Dtype::UnsignedInteger16 => { pretty_assertions::assert_eq!(lvalue.as_vec_u16().unwrap(), rvalue.as_vec_u16().unwrap()); },
                    Dtype::UnsignedInteger32 => { pretty_assertions::assert_eq!(lvalue.as_vec_u32().unwrap(), rvalue.as_vec_u32().unwrap()); },
                    Dtype::UnsignedInteger64 => { pretty_assertions::assert_eq!(lvalue.as_vec_u64().unwrap(), rvalue.as_vec_u64().unwrap()); },
                    Dtype::SignedInteger8 => { pretty_assertions::assert_eq!(lvalue.as_vec_i8().unwrap(), rvalue.as_vec_i8().unwrap()); },
                    Dtype::SignedInteger16 => { pretty_assertions::assert_eq!(lvalue.as_vec_i16().unwrap(), rvalue.as_vec_i16().unwrap()); },
                    Dtype::SignedInteger32 => { pretty_assertions::assert_eq!(lvalue.as_vec_i32().unwrap(), rvalue.as_vec_i32().unwrap()); },
                    Dtype::SignedInteger64 => { pretty_assertions::assert_eq!(lvalue.as_vec_i64().unwrap(), rvalue.as_vec_i64().unwrap()); },
                    Dtype::Float32 => { pretty_assertions::assert_eq!(lvalue.as_vec_f32().unwrap(), rvalue.as_vec_f32().unwrap()); },
                    Dtype::Float64 => { pretty_assertions::assert_eq!(lvalue.as_vec_f64().unwrap(), rvalue.as_vec_f64().unwrap()); }, 
                    Dtype::Str => { unreachable!("String array"); }, 
                }
            } else {
                match lvalue.get_dtype() {
                    Dtype::Byte => { pretty_assertions::assert_eq!(lvalue.as_u8().unwrap(), rvalue.as_u8().unwrap()); },
                    Dtype::UnsignedInteger16 => { pretty_assertions::assert_eq!(lvalue.as_u16().unwrap(), rvalue.as_u16().unwrap()); },
                    Dtype::UnsignedInteger32 => { pretty_assertions::assert_eq!(lvalue.as_u32().unwrap(), rvalue.as_u32().unwrap()); },
                    Dtype::UnsignedInteger64 => { pretty_assertions::assert_eq!(lvalue.as_u64().unwrap(), rvalue.as_u64().unwrap()); },
                    Dtype::SignedInteger8 => { pretty_assertions::assert_eq!(lvalue.as_i8().unwrap(), rvalue.as_i8().unwrap()); },
                    Dtype::SignedInteger16 => { pretty_assertions::assert_eq!(lvalue.as_i16().unwrap(), rvalue.as_i16().unwrap()); },
                    Dtype::SignedInteger32 => { pretty_assertions::assert_eq!(lvalue.as_i32().unwrap(), rvalue.as_i32().unwrap()); },
                    Dtype::SignedInteger64 => { pretty_assertions::assert_eq!(lvalue.as_i64().unwrap(), rvalue.as_i64().unwrap()); },
                    Dtype::Float32 => { pretty_assertions::assert_eq!(lvalue.as_f32().unwrap(), rvalue.as_f32().unwrap()); },
                    Dtype::Float64 => { pretty_assertions::assert_eq!(lvalue.as_f64().unwrap(), rvalue.as_f64().unwrap()); }, 
                    Dtype::Str => { pretty_assertions::assert_eq!(lvalue.as_string().unwrap(), rvalue.as_string().unwrap()); }, 
                }
            }
        }
    }

    #[test]
    fn multiple_members_ok() {
        let text = "foo: u32, bar: f32[10], baz: string";
        let dspec = DesignationSpecification::from_text(text);
        assert_eq!(
            dspec,
            Ok(DesignationSpecification{members: vec![
                MemberSpecification::from_parts(
                    "foo", &Sizing::Singleton, &Dtype::UnsignedInteger32,
                ),
                MemberSpecification::from_parts(
                    "bar", &Sizing::Fixed(10), &Dtype::Float32,
                ),
                MemberSpecification::from_parts(
                    "baz", &Sizing::Singleton, &Dtype::Str,
                ),
            ]})
        );
    }

    #[test]
    fn simple_ok() {
        let text  = "foo: u32, bar: i32";
        let dspec = DesignationSpecification::from_text(text).unwrap(); 
        let expected: DataMap = HashMap::from([
            ("foo", make_dyn_box(10_u32)),
            ("bar", make_dyn_box(-10_i32)),
        ]);
        let buffer = expected.get("foo").unwrap().as_u32().unwrap().to_le_bytes().iter()
            .chain(expected.get("bar").unwrap().as_i32().unwrap().to_le_bytes().iter())
            .copied()
            .collect::<Vec<u8>>();
        let result = dspec.interpret(&buffer).unwrap();
        compare_hashmap(&expected, &result);
    }

    #[test]
    fn simple_fixed_vec_ok() {
        let text  = "foo: u32[3], bar: string";
        let dspec = DesignationSpecification::from_text(text).unwrap(); 
        let expected: DataMap = HashMap::from([
            ("foo", make_dyn_box(vec![2_u32, 10_u32, 0xDEADBEEF_u32])),
            ("bar", make_dyn_box(test_utils::crab_emoji())),
        ]);
        let buffer = expected.get("foo").unwrap().as_vec_u32().unwrap().as_buffer().iter()
            .chain(expected.get("bar").unwrap().as_string().unwrap().as_buffer().iter())
            .copied()
            .collect::<Vec<u8>>();
        let result = dspec.interpret(&buffer).unwrap();
        compare_hashmap(&expected, &result); 
    }

    #[test]
    fn simple_dynamic_vec_ok() {
        let text  = "foo: u32[], bar: string";
        let dspec = DesignationSpecification::from_text(text).unwrap(); 
        let expected: DataMap = HashMap::from([
            ("foo", make_dyn_box(vec![2_u32, 10_u32, 0xDEADBEEF_u32])),
            ("bar", make_dyn_box(test_utils::crab_emoji())),
        ]);
        let buffer = [0x03, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00].iter()
            .chain(expected.get("foo").unwrap().as_vec_u32().unwrap().as_buffer().iter())
            .chain(expected.get("bar").unwrap().as_string().unwrap().as_buffer().iter())
            .copied()
            .collect::<Vec<u8>>();
        let result = dspec.interpret(&buffer).unwrap();
        compare_hashmap(&expected, &result); 
    }

    #[test]
    fn interpret_u8_ok() {
        let text  = "foo: u8";
        let dspec = DesignationSpecification::from_text(text).unwrap(); 
        let val: u8 = 10;
        let result = dspec.interpret(val.to_le_bytes().as_ref());
        let result_val = result.unwrap().get("foo").unwrap().as_u8().unwrap();
        assert_eq!(val, result_val);
    } 
}
