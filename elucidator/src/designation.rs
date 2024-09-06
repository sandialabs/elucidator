use std::collections::HashMap;
use std::io::{Cursor, Read};

use crate::{
    error::*,
    member::{MemberSpecification, Sizing, Dtype},
    parsing,
    util::Buffer,
    validating,
    value::{DataValue, LeBufferRead},
    representable::Representable,
};

use elucidator_macros::make_dtype_interpreter;

type Result<T, E = ElucidatorError> = std::result::Result<T, E>;

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

fn get_val_from_buf<T: Representable + LeBufferRead>(buffer: &mut Buffer) -> Result<T> {
    T::get_one_le(&buffer.grab(T::bytes_needed(1))?)
}

fn get_n_vals_from_buf<T: Representable + LeBufferRead>(buffer: &mut Buffer, n: usize) -> Result<Vec<T>> {
    T::get_n_le(&buffer.grab(T::bytes_needed(n))?, n)
}

fn get_box_dtype(buffer: &mut Buffer, dt: &Dtype) -> Result<Box<dyn Representable>> {
    let b: Box<dyn Representable> = match dt {
            Dtype::Byte => Box::new(get_val_from_buf::<u8>(buffer)?),
            Dtype::UnsignedInteger16 => {
                Box::new(get_val_from_buf::<u16>(buffer)?)
            },
            Dtype::UnsignedInteger32 => {
                Box::new(get_val_from_buf::<u32>(buffer)?)
            },
            Dtype::UnsignedInteger64 => {
                Box::new(get_val_from_buf::<u64>(buffer)?)
            },
            Dtype::SignedInteger8 => {
                Box::new(get_val_from_buf::<i8>(buffer)?)
            },
            Dtype::SignedInteger16 => {
                Box::new(get_val_from_buf::<i16>(buffer)?)
            },
            Dtype::SignedInteger32 => {
                Box::new(get_val_from_buf::<i32>(buffer)?)
            },
            Dtype::SignedInteger64 => {
                Box::new(get_val_from_buf::<i64>(buffer)?)
            },
            Dtype::Float32 => {
                Box::new(get_val_from_buf::<f32>(buffer)?)
            },
            Dtype::Float64 => {
                Box::new(get_val_from_buf::<f64>(buffer)?)
            },
            Dtype::Str => {
                Box::new(get_string_from_buf(buffer)?)
            },
    };
    Ok(b)
}

fn get_box_n_dtype(buffer: &mut Buffer, n: usize, dt: &Dtype) -> Result<Box<dyn Representable>> {
    let b: Box<dyn Representable> = match dt {
        Dtype::Byte => Box::new(get_n_vals_from_buf::<u8>(buffer, n)?),
        Dtype::UnsignedInteger16 => Box::new(get_n_vals_from_buf::<u16>(buffer, n)?),
        Dtype::UnsignedInteger32 => Box::new(get_n_vals_from_buf::<u32>(buffer, n)?),
        Dtype::UnsignedInteger64 => Box::new(get_n_vals_from_buf::<u64>(buffer, n)?),
        Dtype::SignedInteger8 => Box::new(get_n_vals_from_buf::<i8>(buffer, n)?),
        Dtype::SignedInteger16 => Box::new(get_n_vals_from_buf::<i16>(buffer, n)?),
        Dtype::SignedInteger32 => Box::new(get_n_vals_from_buf::<i32>(buffer, n)?),
        Dtype::SignedInteger64 => Box::new(get_n_vals_from_buf::<i64>(buffer, n)?),
        Dtype::Float32 => Box::new(get_n_vals_from_buf::<f32>(buffer, n)?),
        Dtype::Float64 => Box::new(get_n_vals_from_buf::<f64>(buffer, n)?),
        Dtype::Str => { unreachable!("Can't fetch arrays of strings"); },
    };
    Ok(b)
}


fn get_string_from_buf(buffer: &mut Buffer) -> Result<String> {
    let size = u64::from_le_bytes(buffer.grab(8)?.try_into().unwrap());
    let databuf = buffer.grab(size as usize)?;
    match String::from_utf8(databuf) {
        Ok(s) => Ok(s),
        Err(e) => Err(ElucidatorError::FromUtf8{ source: e }),
    }
}

// DON'T USE THIS EXCEPT INSIDE OF INTERPRETING ENUMS
fn get_singleton_from_buf(buffer: &mut Buffer, dt: &Dtype) -> Result<DataValue> {
    match dt {
        Dtype::Byte => {
            let buf = buffer.grab(u8::bytes_needed(1))?;
            Ok(DataValue::Byte(u8::get_one_le(&buf)?))
        },
        Dtype::UnsignedInteger16 => {
            let buf = buffer.grab(u16::bytes_needed(1))?;
            Ok(DataValue::UnsignedInteger16(
                u16::get_one_le(&buf)?
            ))
        },
        Dtype::UnsignedInteger32 => {
            let buf = buffer.grab(u32::bytes_needed(1))?;
            Ok(DataValue::UnsignedInteger32(
                u32::get_one_le(&buf)?
            ))
        },
        Dtype::UnsignedInteger64 => {
            let buf = buffer.grab(u64::bytes_needed(1))?;
            Ok(DataValue::UnsignedInteger64(
                u64::get_one_le(&buf)?
            ))
        },
        Dtype::SignedInteger8 => {
            let buf = buffer.grab(i8::bytes_needed(1))?;
            Ok(DataValue::SignedInteger8(i8::get_one_le(&buf)?))
        },
        Dtype::SignedInteger16 => {
            let buf = buffer.grab(i16::bytes_needed(1))?;
            Ok(DataValue::SignedInteger16(
                i16::get_one_le(&buf)?
            ))
        },
        Dtype::SignedInteger32 => {
            let buf = buffer.grab(i32::bytes_needed(1))?;
            Ok(DataValue::SignedInteger32(
                i32::get_one_le(&buf)?
            ))
        },
        Dtype::SignedInteger64 => {
            let buf = buffer.grab(i64::bytes_needed(1))?;
            Ok(DataValue::SignedInteger64(
                i64::get_one_le(&buf)?
            ))
        },
        Dtype::Float32 => {
            let buf = buffer.grab(f32::bytes_needed(1))?;
            Ok(DataValue::Float32(
                f32::get_one_le(&buf)?
            ))
        },
        Dtype::Float64 => {
            let buf = buffer.grab(f64::bytes_needed(1))?;
            Ok(DataValue::Float64(
                f64::get_one_le(&buf)?
            ))
        },
        Dtype::Str => {
            let string_length = u64::from_le_bytes(buffer.grab(8)?.try_into().unwrap());
            let string_contents = buffer.grab(string_length as usize)?;
            let s = match String::from_utf8(string_contents) {
                Ok(o) => o,
                Err(e) => Err(ElucidatorError::FromUtf8{ source: e })?,
            };
            Ok(DataValue::Str(s))
        },
    }
}

// DON'T USE THIS EXCEPT INSIDE OF INTERPRETING ENUMS
fn get_array_from_buf(buffer: &mut Buffer, dt: &Dtype, items_to_read: usize) -> Result<DataValue> {
    match dt {
        Dtype::Byte => { 
            let buf = &buffer.grab(u8::bytes_needed(items_to_read))?;
            Ok(DataValue::ByteArray(u8::get_n_le(&buf, items_to_read)?))
        },
        Dtype::UnsignedInteger16 => {
            let buf = &buffer.grab(u16::bytes_needed(items_to_read))?;
            Ok(DataValue::UnsignedInteger16Array(
                u16::get_n_le(buf, items_to_read)?
            ))
        },
        Dtype::UnsignedInteger32 => {
            let buf = &buffer.grab(u32::bytes_needed(items_to_read))?;
            Ok(DataValue::UnsignedInteger32Array(
                u32::get_n_le(buf, items_to_read)?
            ))
        },
        Dtype::UnsignedInteger64 => {
            let buf = &buffer.grab(u64::bytes_needed(items_to_read))?;
            Ok(DataValue::UnsignedInteger64Array(
                u64::get_n_le(buf, items_to_read)?
            ))
        },
        Dtype::SignedInteger8 => { 
            let buf = &buffer.grab(i8::bytes_needed(items_to_read))?;
            Ok(DataValue::SignedInteger8Array(
                    i8::get_n_le(buf, items_to_read)?
            ))
        },
        Dtype::SignedInteger16 => {
            let buf = &buffer.grab(i16::bytes_needed(items_to_read))?;
            Ok(DataValue::SignedInteger16Array(
                i16::get_n_le(buf, items_to_read)?
            ))
        },
        Dtype::SignedInteger32 => {
            let buf = &buffer.grab(i32::bytes_needed(items_to_read))?;
            Ok(DataValue::SignedInteger32Array(
                i32::get_n_le(buf, items_to_read)?
            ))
        },
        Dtype::SignedInteger64 => {
            let buf = &buffer.grab(i64::bytes_needed(items_to_read))?;
            Ok(DataValue::SignedInteger64Array(
                i64::get_n_le(buf, items_to_read)?
            ))
        },
        Dtype::Float32 => {
            let buf = &buffer.grab(f32::bytes_needed(items_to_read))?;
            Ok(DataValue::Float32Array(
                f32::get_n_le(buf, items_to_read)?
            ))
        },
        Dtype::Float64 => {
            let buf = &buffer.grab(f64::bytes_needed(items_to_read))?;
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
        let mut buf = Buffer::new(buffer);
        for member in &self.members {
            let val: Box<dyn Representable> = match member.sizing {
                Sizing::Singleton => {
                    get_box_dtype(&mut buf, &member.dtype)?
                },
                Sizing::Fixed(n) => {
                    let n = n as usize;
                    get_box_n_dtype(&mut buf, n, &member.dtype)?
                },
                Sizing::Dynamic => {
                    let n = u64::from_le_bytes(
                        buf.grab(8)?.try_into().unwrap()
                    ) as usize;
                    get_box_n_dtype(&mut buf, n, &member.dtype)?
                },
            };
            map.insert(member.identifier.as_str(), val);
        }
        Ok(map)
    }

    pub fn interpret_enum(&self, buffer: &[u8]) -> Result<HashMap<&str, DataValue>> {
        let mut map = HashMap::new();
        let mut buf = Buffer::new(buffer);
        for member in &self.members {
            let member_name = member.identifier.as_str().clone();
            let value = match member.sizing {
                Sizing::Singleton => {
                    get_singleton_from_buf(&mut buf, &member.dtype)? 
                },
                Sizing::Fixed(n) => {
                    get_array_from_buf(&mut buf, &member.dtype, n as usize)?
                },
                Sizing::Dynamic => {
                    let n = u64::from_le_bytes(buf.grab(8)?.try_into().unwrap());
                    get_array_from_buf(&mut buf, &member.dtype, n as usize)?
                }
            };
            map.insert(member_name, value);
        }
        Ok(map)
    }
}

#[cfg(test)]
mod test {
    use std::collections::HashSet;

    use super::*;
    use crate::{member::{Dtype, Sizing}, test_utils, value::DataValue};
    use rand::{random, Rng};
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

    fn random_data_value(dt: &Dtype, sizing: &Sizing) -> DataValue {
        let items = match sizing {
            Sizing::Singleton => { 1 },
            Sizing::Fixed(n) => { *n },
            Sizing::Dynamic => { (random::<u8>() % 100 + 1) as u64 },
        };
		match dt {
			Dtype::Byte => {
				if sizing == &Sizing::Singleton {
					DataValue::Byte(random())
				} else {
					DataValue::ByteArray((0..items).map(|_| random::<u8>()).collect())
				}
			},
			Dtype::UnsignedInteger16 => {
				if sizing == &Sizing::Singleton {
					DataValue::UnsignedInteger16(random())
				} else {
					DataValue::UnsignedInteger16Array((0..items).map(|_| random::<u16>()).collect())
				}
			},
			Dtype::UnsignedInteger32 => {
				if sizing == &Sizing::Singleton {
					DataValue::UnsignedInteger32(random())
				} else {
					DataValue::UnsignedInteger32Array((0..items).map(|_| random::<u32>()).collect())
				}
			},
			Dtype::UnsignedInteger64 => {
				if sizing == &Sizing::Singleton {
					DataValue::UnsignedInteger64(random())
				} else {
					DataValue::UnsignedInteger64Array((0..items).map(|_| random::<u64>()).collect())
				}
			},
			Dtype::SignedInteger8 => {
				if sizing == &Sizing::Singleton {
					DataValue::SignedInteger8(random())
				} else {
					DataValue::SignedInteger8Array((0..items).map(|_| random::<i8>()).collect())
				}
			},
			Dtype::SignedInteger16 => {
				if sizing == &Sizing::Singleton {
					DataValue::SignedInteger16(random())
				} else {
					DataValue::SignedInteger16Array((0..items).map(|_| random::<i16>()).collect())
				}
			},
			Dtype::SignedInteger32 => {
				if sizing == &Sizing::Singleton {
					DataValue::SignedInteger32(random())
				} else {
					DataValue::SignedInteger32Array((0..items).map(|_| random::<i32>()).collect())
				}
			},
			Dtype::SignedInteger64 => {
				if sizing == &Sizing::Singleton {
					DataValue::SignedInteger64(random())
				} else {
					DataValue::SignedInteger64Array((0..items).map(|_| random::<i64>()).collect())
				}
			},
			Dtype::Float32 => {
				if sizing == &Sizing::Singleton {
					DataValue::Float32(random())
				} else {
					DataValue::Float32Array((0..items).map(|_| random::<f32>()).collect())
				}
			},
			Dtype::Float64 => {
				if sizing == &Sizing::Singleton {
					DataValue::Float64(random())
				} else {
					DataValue::Float64Array((0..items).map(|_| random::<f64>()).collect())
				}
			},
			Dtype::Str => {
				let n_chars = random::<u8>() % 10;
				let s = (0..n_chars).map(|_| random::<char>()).collect();
				DataValue::Str(s)
			},
		}
    }

    fn random_sizing() -> Sizing {
        let num = random::<u8>() % 3;
        match num {
            0 => { Sizing::Singleton },
            1 => { Sizing::Fixed((random::<u8>() % 100 + 1) as u64) },
            2 => { Sizing::Dynamic },
            _ => { unreachable!(); }
        }
    }

    fn random_dtype() -> Dtype {
        let num = random::<u8>() % 11; // There are 11 variants in the Dtype enum
        match num {
            0 => Dtype::Byte,
            1 => Dtype::UnsignedInteger16,
            2 => Dtype::UnsignedInteger32,
            3 => Dtype::UnsignedInteger64,
            4 => Dtype::SignedInteger8,
            5 => Dtype::SignedInteger16,
            6 => Dtype::SignedInteger32,
            7 => Dtype::SignedInteger64,
            8 => Dtype::Float32,
            9 => Dtype::Float64,
            10 => Dtype::Str,
            _ => unreachable!(),
        }
    }

   fn random_dtype_sizing() -> (Sizing, Dtype) {
	   let dtype = random_dtype();
	   let sizing = if let Dtype::Str = dtype {
		   Sizing::Singleton
	   } else {
		   random_sizing()
	   };
	   (sizing, dtype)
    }

   fn random_identifier() -> String {
       let mut rng = rand::thread_rng();
       let length = (random::<u8>() % 5) + 1;
       (0..length)
           .map(|_| (rng.gen_range(b'a'..=b'z') as char))
           .collect()
    }

    fn random_member_specification() -> MemberSpecification {
        let (sizing, dtype) = random_dtype_sizing();
        let identifier = random_identifier();
        MemberSpecification {
            identifier,
            sizing,
            dtype,
        }
    }

    fn random_designation_specification() -> DesignationSpecification {
        let num_members = random::<u8>() % 20 + 1;
        let members = loop {
            let candidates = (0..num_members)
                .map(|_| random_member_specification())
                .collect::<Vec<_>>();
            let unique_names = candidates.iter()
                .map(|x| x.identifier.clone())
                .collect::<HashSet<_>>();
            if candidates.len() == unique_names.len() {
                break candidates
            }
        };
        DesignationSpecification { members }
    }

    fn generate_random_designation_specification_data(designation_spec: &DesignationSpecification) -> HashMap<&str, DataValue> {
        let mut data_map = HashMap::new();

        for member in &designation_spec.members {
            let data_value = random_data_value(&member.dtype, &member.sizing);
            data_map.insert(member.identifier.as_str(), data_value);
        }

        data_map
    }


    fn into_blob(dv: &DataValue, sizing: &Sizing) -> Vec<u8> {
        let mut buffer = Vec::new();

        if let Sizing::Dynamic = sizing {
            let num_elements = match dv {
                DataValue::ByteArray(v) => v.len() as u64,
                DataValue::UnsignedInteger16Array(v) => v.len() as u64,
                DataValue::UnsignedInteger32Array(v) => v.len() as u64,
                DataValue::UnsignedInteger64Array(v) => v.len() as u64,
                DataValue::SignedInteger8Array(v) => v.len() as u64,
                DataValue::SignedInteger16Array(v) => v.len() as u64,
                DataValue::SignedInteger32Array(v) => v.len() as u64,
                DataValue::SignedInteger64Array(v) => v.len() as u64,
                DataValue::Float32Array(v) => v.len() as u64,
                DataValue::Float64Array(v) => v.len() as u64,
                _ => {
                    unreachable!("Only arrays should have dynamic sizing");
                },
            };
            buffer.extend_from_slice(&num_elements.to_le_bytes());
        }

        buffer.extend_from_slice(&dv.as_buffer());
        buffer
    }

    fn generate_designation_and_perform_round_trip() {
         let designation = random_designation_specification();
         let n_data = random::<u8>() % 50;
         let data_vec: Vec<HashMap<&str, DataValue>> = (0..n_data)
             .map(|_| generate_random_designation_specification_data(&designation))
             .collect();
         for datum in &data_vec {
            let blob_vec: Vec<Vec<u8>> = designation.members.iter()
                .map(|member| {
                    let dv = datum.get(member.identifier.as_str()).unwrap();
                    let sizing = &member.sizing;
                    into_blob(&dv, sizing)
                })
                .collect();
             let buffer: Vec<u8> = blob_vec.iter()
                 .flat_map(|x| x.iter())
                 .copied()
                 .collect();
             let map = designation.interpret_enum(&buffer);
             let dr: Result<HashMap<&str, DataValue>> = Ok(datum.clone());
             pretty_assertions::assert_eq!(
                 map,
                 dr,
                 "{designation:#?}\nBuffer size {}", buffer.len()
             );
         }
     }
 
    #[test]
    fn compare_dv_hm() {
        let left = HashMap::from([
            ("foo", DataValue::Byte(9)),
            ("bar", DataValue::Float32Array(vec![-5.0, -10.0, 3.14])),
        ]);
        let right = HashMap::from([
            ("foo", DataValue::Byte(9)),
            ("bar", DataValue::Float32Array(vec![-5.0, -10.0, 3.14])),
        ]);
        pretty_assertions::assert_eq!(
            left,
            right,
        );
    }

    #[test]
    fn simple_interpret_enum() {
        let hm = HashMap::from([
            ("foo", DataValue::Byte(9)),
            ("bar", DataValue::Float32Array(vec![-5.0, -10.0, 3.14])),
        ]);
        let buff_foo = hm.get("foo").unwrap().as_buffer();
        let buff_bar = hm.get("bar").unwrap().as_buffer();
        let buffer: Vec<u8> = buff_foo.iter()
            .chain(buff_bar.iter())
            .copied()
            .collect();
        let designation = DesignationSpecification::from_text("foo: u8, bar: f32[3]").unwrap();
        let result = designation.interpret_enum(&buffer);
        pretty_assertions::assert_eq!(
            result,
            Ok(hm),
        );
    }

    #[test]
    fn complex_interpret_enum() {
        let foo_vec: Vec<i16> = vec![-1, 2, 1025];
        let bar_vec: Vec<f64> = vec![3.1415, 2.71];
        let baz_vec: Vec<i8> = vec![-3, 4, -5, 6];
        let hm = HashMap::from([
            ("foo", DataValue::SignedInteger16Array(foo_vec.clone())),
            ("bar", DataValue::Float64Array(bar_vec)),
            ("baz", DataValue::SignedInteger8Array(baz_vec.clone())),
        ]);
        let foo_size_buf = (foo_vec.len() as u64).to_le_bytes();
        let baz_size_buf = (baz_vec.len() as u64).to_le_bytes();
        let buffer: Vec<u8> = foo_size_buf.iter()
            .chain(hm.get("foo").unwrap().as_buffer().iter())
            .chain(hm.get("bar").unwrap().as_buffer().iter())
            .chain(baz_size_buf.iter())
            .chain(hm.get("baz").unwrap().as_buffer().iter())
            .copied()
            .collect();
        let designation = DesignationSpecification::from_text(
            "foo: i16[], bar: f64[2], baz: i8[]"
        ).unwrap();
        let result = designation.interpret_enum(&buffer);
        pretty_assertions::assert_eq!(
            result,
            Ok(hm),
        );
    }

    #[test]
    fn property_test_interpret_enum() {
        for _ in 0..1000 {
            generate_designation_and_perform_round_trip()
        }
    }
}
