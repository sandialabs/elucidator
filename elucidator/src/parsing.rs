use crate::{error::*, token::*};

type Result<T, E = ElucidatorError> = std::result::Result<T, E>;

#[derive(Debug, PartialEq)]
pub(crate) struct IdentifierParserOutput<'a> {
    identifier: Option<IdentifierToken<'a>>,
    errors: Vec<ElucidatorError>,
}

#[derive(Debug, PartialEq)]
pub(crate) struct MemberSpecParserOutput<'a> {
    identifier: Option<IdentifierToken<'a>>,
    dtype: Option<DtypeToken<'a>>,
    sizing: Option<SizeToken<'a>>,
    errors: Vec<ElucidatorError>,
}

pub fn get_identifier<'a>(data: &'a str, start_col: usize) -> IdentifierParserOutput<'a> {
    let identifier_end = start_col + data.len();
    let itoken = IdentifierToken {
        data: TokenData::new(data, start_col, identifier_end),
    };
    IdentifierParserOutput {
        identifier: Some(itoken),
        errors: Vec::new(),
    }
}

pub fn get_memberspec<'a>(data: &'a str, start_col: usize) -> MemberSpecParserOutput<'a> {
    let mut errors = Vec::new();

    todo!();

    MemberSpecParserOutput {
        identifier: None,
        dtype: None,
        sizing: None,
        errors,
    }
}

// TODO: REMOVE
pub fn validate_identifier(_: &str) -> Result<()> {
    unimplemented!();
}

pub fn is_valid_identifier_char(_: char) -> bool {
    unimplemented!();
}

pub fn ascii_trimmed_or_err(_: &str) -> Result<&str> {
    unimplemented!();
}

pub fn validated_trimmed_or_err(_: &str) -> Result<&str> {
    unimplemented!();
}

// pub(crate) struct MemberSpecParserOutput<'a> {
//     identifier: Option<IdentifierToken>,
//     dtype: Option<DtypeToken>,
//     sizing: Option<SizeToken>,
// }
// 
// impl <'a> MemberSpecParserOutput<'a> {
//     fn 


#[cfg(test)]
mod test {
    use super::*;

    mod identifier {
        use super::*;

        #[test]
        fn ok() {
            let text = "valid_identifier";
            let output = get_identifier(text, 0);
            let data = TokenData::new(text, 0, text.len());
            let itoken = IdentifierToken { data };
            assert_eq!(
                output,
                IdentifierParserOutput {
                    identifier: Some(itoken),
                    errors: Vec::new(),
                }
            );
        }
    }

    // Tests marked "invalid" are invalid according to the standard, but are parseable.
    mod member_spec {
        use super::*;
        use rand::prelude::random;
    
        /// For making sure text with no whitespace works
        fn run_ok_simple(ident: &str, dtype: &str, sizing: Option<&str>) {
            let text = if let Some(size) = sizing {
                format!("{ident}:{dtype}[{size}]")
            } else {
                format!("{ident}:{dtype}")
            };
            let output = get_memberspec(&text, 0);
            let mut curr_start = 0;
            let mut curr_end = ident.len();
            let td_ident = TokenData::new(ident, curr_start, curr_end);
            // colon
            curr_start = curr_end;
            curr_end += 1;
            // starting dtype
            curr_start = curr_end;
            curr_end += dtype.len();
            let td_dtype = TokenData::new(dtype, curr_start, curr_end);
            let t_identifier = IdentifierToken { data: td_ident };
            let t_dtype = DtypeToken { data: td_dtype };
            // Make a string so that the string slice lives to the end of the function
            let mut size_string = String::new();
            let t_sizing = if let Some(size) = sizing {
                // left bracket
                curr_start = curr_end;
                curr_end += 1;
                // starting sizing
                curr_start = curr_end;
                curr_end += size.len();
                size_string = String::from(size);
                Some(SizeToken {
                    data: TokenData::new(&size_string, curr_start, curr_end)
                })
            } else { None };

            assert_eq!(output.identifier, Some(t_identifier));
            assert_eq!(output.dtype, Some(t_dtype));
            assert_eq!(output.sizing, t_sizing);
            assert!(output.errors.is_empty());
        }

        /// Get the set of whitespace characters
        fn get_whitespace_chars() -> Vec<char> {
            (u8::MIN..u8::MAX)
                .map(|x| x as char)
                .filter(|x| x.is_whitespace())
                .collect()
        }

        /// Get random whitespace
        fn get_random_whitespace(num_chars: usize) -> String {
            let whitespace_chars = get_whitespace_chars();
            (0..num_chars)
                .map(|_| random::<usize>() % whitespace_chars.len())
                .map(|x| String::from(whitespace_chars[x]))
                .collect::<Vec<String>>()
                .join("")
        }

        /// Produce a potential whitespace fill
        fn fill() -> String {
            get_random_whitespace(random::<usize>() % 4)
        }

        /// Run a single test case of whitespace insertion
        fn run_ok_whitespace(ident: &str, dtype: &str, sizing: Option<&str>) {
            let lident = fill();
            let rident = fill();
            let ldtype = fill();
            let rdtype = fill();
            let lsizing = fill();
            let rsizing = fill();
            let end = fill();

            let text = if let Some(size) = sizing {
                format!("{lident}{ident}{rident}:{ldtype}{dtype}{rdtype}[{lsizing}{size}{rsizing}]{end}")
            } else {
                format!("{lident}{ident}{rident}:{ldtype}{dtype}{rdtype}")
            };
            let output = get_memberspec(&text, 0);
            // left ident whitespace
            let mut curr_start = 0;
            let mut curr_end = lident.len();
            // ident
            curr_start = curr_end;
            curr_end += ident.len();
            let td_ident = TokenData::new(ident, curr_start, curr_end);
            // right ident whitespace
            curr_start = curr_end;
            curr_end += rident.len();
            // colon
            curr_start = curr_end;
            curr_end += 1;
            // left dtype whitespace
            curr_start = curr_end;
            curr_end += ldtype.len();
            // starting dtype
            curr_start = curr_end;
            curr_end += dtype.len();
            let td_dtype = TokenData::new(dtype, curr_start, curr_end);
            let t_identifier = IdentifierToken { data: td_ident };
            let t_dtype = DtypeToken { data: td_dtype };
            // right dtype whitespace
            curr_start = curr_end;
            curr_end += rdtype.len();
            // Make a string so that the string slice lives to the end of the function
            let mut size_string = String::new();
            let t_sizing = if let Some(size) = sizing {
                // left bracket
                curr_start = curr_end;
                curr_end += 1;
                // left sizing whitespace
                curr_start = curr_end;
                curr_end += lsizing.len();
                // starting sizing
                curr_start = curr_end;
                curr_end += size.len();
                size_string = String::from(size);
                Some(SizeToken {
                    data: TokenData::new(&size_string, curr_start, curr_end)
                })
            } else { None };

            assert_eq!(output.identifier, Some(t_identifier));
            assert_eq!(output.dtype, Some(t_dtype));
            assert_eq!(output.sizing, t_sizing);
            assert!(output.errors.is_empty());
        }

        #[test]
        fn whitespace_property_test() {
            for _ in (0..500) {
                run_ok_whitespace("foo", "u8", None);
            }
            for _ in (0..500) {
                run_ok_whitespace("foo", "u8", Some(""));
            }
            for _ in (0..500) {
                run_ok_whitespace("foo", "u8", Some("1000"));
            }
        }

        #[test]
        fn ok_invalid_string() {
            run_ok_simple("5ever", "silly", None);
        }
        #[test]
        fn ok_valid_string() {
            run_ok_simple("animal", "string", None);
        }
        #[test]
        fn ok_invalid_fixed_array_size() {
            run_ok_simple("myarr", "f32", Some("cat"));
        }
        #[test]
        fn ok_invalid_fixed_array_type() {
            run_ok_simple("myarr", "cat", Some("5"));
        }
        #[test]
        fn ok_valid_fixed_array() {
            run_ok_simple("myarr", "f32", Some("5"));
        }
        #[test]
        fn ok_invalid_dyn_array() {
            run_ok_simple("myarr", "cat", Some(""));
        }
        #[test]
        fn ok_valid_dyn_array() {
            run_ok_simple("myarr", "f32", Some(""));
        }
        #[test]
        fn ok_no_whitespace() {
            run_ok_simple("myarr", "f32", Some("5"));
        }
    }
}
