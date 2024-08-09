use crate::{error::*, token::*};

type Result<T, E = ElucidatorError> = std::result::Result<T, E>;

#[derive(Debug, PartialEq)]
pub(crate) struct WordParserOutput<'a> {
    word: Option<TokenData<'a>>,
    errors: Vec<ElucidatorError>,
}

#[derive(Debug, PartialEq)]
pub(crate) struct IdentifierParserOutput<'a> {
    identifier: Option<IdentifierToken<'a>>,
    errors: Vec<ElucidatorError>,
}

#[derive(Debug, PartialEq)]
pub(crate) struct DtypeParserOutput<'a> {
    dtype: Option<DtypeToken<'a>>,
    errors: Vec<ElucidatorError>,
}

#[derive(Debug, PartialEq)]
pub(crate) struct SizingParserOutput<'a> {
    sizing: Option<SizingToken<'a>>,
    errors: Vec<ElucidatorError>,
}
#[derive(Debug, PartialEq)]
pub(crate) struct TypeSpecParserOutput<'a> {
    dtype: Option<DtypeToken<'a>>,
    sizing: Option<SizingToken<'a>>,
    errors: Vec<ElucidatorError>,
    is_singleton: bool,
}
#[derive(Debug, PartialEq)]
pub(crate) struct MemberSpecParserOutput<'a> {
    identifier: Option<IdentifierToken<'a>>,
    dtype: Option<DtypeToken<'a>>,
    sizing: Option<SizingToken<'a>>,
    errors: Vec<ElucidatorError>,
}

pub fn get_identifier<'a>(data: &'a str, start_col: usize) -> IdentifierParserOutput {
    let word_output = get_word(data, start_col);
    let identifier = if let Some(word) = word_output.word {
        Some(IdentifierToken{ data: word })
    } else {
        None
    };
    let errors = word_output.errors;
    IdentifierParserOutput {
        identifier,
        errors,
    }
}

pub fn get_dtype<'a>(data: &'a str, start_col: usize) -> DtypeParserOutput<'a> {
    let word_output = get_word(data, start_col);
    let dtype = if let Some(word) = word_output.word {
        Some(DtypeToken{ data: word })
    } else {
        None
    };
    let errors = word_output.errors;
    DtypeParserOutput {
        dtype,
        errors,
    }
}

pub fn get_sizing<'a>(data: &'a str, start_col: usize) -> SizingParserOutput<'a> {
    if data.chars().all(|x| x.is_whitespace()) {
        let data_len = data.len();
        let last_slice = if data_len == 0 {
            &data[0..0]
        } else {
            let (last, _) = data.char_indices().last().unwrap();
            &data[last..last]
        };
        let pos = start_col + data_len;
        let stoken = SizingToken {
            data: TokenData::new(last_slice, pos, pos),
        };
        SizingParserOutput {
            sizing: Some(stoken),
            errors: Vec::new(),
        }
    }
    else {
        let word_output = get_word(data, start_col);
        let sizing = if let Some(word) = word_output.word {
            Some(SizingToken{ data: word })
        } else {
            unreachable!("get_sizing dispatched when singleton should have been found by get_typespec");
        };
        let errors = word_output.errors;
        SizingParserOutput {
            sizing,
            errors,
        }
    }
}

pub fn get_word<'a>(data: &'a str, start_col: usize) -> WordParserOutput<'a> {
    let word;
    let mut errors = Vec::new();
    let id_start = data.char_indices().find(|(_, x)| !x.is_whitespace());
    if id_start.is_none() {
        errors.push(
            ElucidatorError::Parsing {
                offender: data.to_string(),
                reason: ParsingFailure::UnexpectedEndOfExpression,
            }
        );
    };
    let (id_byte_start, _) = id_start.unwrap();
    if errors.is_empty() {
        // TODO:
        // This is our actual error:
        // id_start is defined as the index of the first whitespace *character*
        // We are then using that index on the *bytes* of data
        // let id_end = if let Some(pos) = data[id_start..].chars().position(|x| x.is_whitespace()) {
        // let id_end = if let Some(pos) = data.chars().skip(id_start).position(|x| x.is_whitespace())
        let id_byte_end = if let Some((pos, _)) = data[id_byte_start..].char_indices().find(|(_, x)| x.is_whitespace())
        {
            pos + id_byte_start
        } else {
            data.len()
        };
        let id_char_start = &data[..id_byte_start].chars().count();
        let id_char_end = &data[..id_byte_end].chars().count();
        word = Some(TokenData::new(
            &data[id_byte_start..id_byte_end],
            id_char_start + start_col,
            id_char_end + start_col
        ));
    }
    else {
        word = None;
    }

    WordParserOutput {
        word,
        errors,
    }
}


pub fn get_typespec<'a>(data: &'a str, start_col: usize) -> TypeSpecParserOutput {
    let dtype ;
    let sizing ;
    let is_singleton;
    let end_of_dtype ;
    let mut errors = Vec::new();
    if let Some((_, contents)) = data.split_once("[") {
        is_singleton = false;
        let lbracket_pos = data.chars().position(|c| c == '[').unwrap();
        end_of_dtype = data.chars().take(lbracket_pos).collect::<String>().len();
        match contents.chars().position(|c| c == ']') {
            Some(rbracket_pos) => {
                let lbracket_byte_pos = data.chars().take(lbracket_pos+1).collect::<String>().len();
                let rbracket_byte_pos = data.chars().take(lbracket_pos + rbracket_pos + 1).collect::<String>().len();
                let byte_start = lbracket_byte_pos;
                let byte_end = rbracket_byte_pos;
                let spo = get_sizing(
                    &data[byte_start..byte_end],
                    start_col + lbracket_pos
                );
                sizing = spo.sizing;
                for error in &spo.errors {
                    errors.push(error.clone());
                }
            },
            None => {
                sizing = None;
                errors.push(
                    ElucidatorError::Parsing {
                        offender: data.to_string(),
                        reason: ParsingFailure::UnexpectedEndOfExpression
                    }
                );
            }
        }
    } else {
        is_singleton = true;
        end_of_dtype = data.len();
        sizing = None;
    }

    let dpo = get_dtype(&data[..end_of_dtype], start_col);
    dtype = dpo.dtype;
    for error in &dpo.errors {
        errors.push(error.clone());
    } 

    TypeSpecParserOutput {
        dtype,
        sizing,
        errors,
        is_singleton,
    }
}

pub fn get_memberspec<'a>(data: &'a str, start_col: usize) -> MemberSpecParserOutput<'a> {
    let mut identifier = None;
    let mut dtype = None;
    let mut sizing = None;
    let mut errors = Vec::new();

    if let Some((left_of_colon, right_of_colon)) = data.split_once(":") {
        let colon_pos = data.chars().position(|c| c == ':').unwrap();
        // Identifier parsing
        let ipo = get_identifier(left_of_colon, start_col);
        identifier = ipo.identifier;
        for error in &ipo.errors {
            errors.push(error.clone());
        }
        // TypeSpec parsing
        let tso = get_typespec(right_of_colon, start_col + colon_pos + 1);
        dtype = tso.dtype;
        sizing = tso.sizing;
        for error in &tso.errors {
            errors.push(error.clone());
        }
    } else {
        errors.push(
            ElucidatorError::Parsing {
                offender: data.to_string().trim().to_string(), 
                reason: ParsingFailure::MissingIdSpecDelimiter
            }
        );
    }

    MemberSpecParserOutput {
        identifier,
        dtype,
        sizing,
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


#[cfg(test)]
mod test {
    use super::*;

    mod word {
        use super::*;

        #[test]
        fn whitespace_ok() {
            // 2 front spaces, 3 back spaces
            let text = "  valid_word   ";
            let output = get_word(text, 0);
            let length = "valid_word".len();
            let token_data = TokenData::new(
                &text[2..(length + 2)],
                2,
                length + 2,
            );
            assert_eq!(
                output,
                WordParserOutput{
                    word: Some(token_data),
                    errors: Vec::new(),
                }
            );
        }

        #[test]
        fn ok() {
            let text = "valid_word";
            let output = get_word(text, 0);
            let data = TokenData::new(text, 0, text.len());
            assert_eq!(
                output,
                WordParserOutput {
                    word: Some(data),
                    errors: Vec::new(),
                }
            );
        }
    }

    mod identifier {
        use super::*;
        
        #[test]
        fn whitespace_ok() {
            // 2 front spaces, 3 back spaces
            let text = "  valid_identifier   ";
            let output = get_identifier(text, 0);
            let length = "valid_identifier".len();
            let itoken = IdentifierToken{data: TokenData::new(
                &text[2..(length + 2)],
                2,
                length + 2,
            )};
            assert_eq!(
                output,
                IdentifierParserOutput {
                    identifier: Some(itoken),
                    errors: Vec::new(),
                }
            );
        }

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
    
    mod dtype {
        use super::*;

        #[test]
        fn whitespace_ok() {
            // 2 front spaces, 3 back spaces
            let text = "  valid_dtype   ";
            let output = get_dtype(text, 0);
            let length = "valid_dtype".len();
            let dtoken = DtypeToken{data: TokenData::new(
                &text[2..(length + 2)],
                2,
                length + 2,
            )};
            assert_eq!(
                output,
                DtypeParserOutput {
                    dtype: Some(dtoken),
                    errors: Vec::new(),
                }
            );
        }

        #[test]
        fn ok() {
            let text = "valid_dtype";
            let output = get_dtype(text, 0);
            let data = TokenData::new(text, 0, text.len());
            let dtoken = DtypeToken { data };
            assert_eq!(
                output,
                DtypeParserOutput {
                    dtype: Some(dtoken),
                    errors: Vec::new(),
                }
            );
        }
    }

    mod sizing {
        use super::*;

        #[test]
        fn fixed_whitespace_ok() {
            // 2 front spaces, 3 back spaces
            let text = "  valid_sizing   ";
            let output = get_sizing(text, 0);
            let length = "valid_sizing".len();
            let stoken = SizingToken{data: TokenData::new(
                &text[2..(length + 2)],
                2,
                length + 2,
            )};
            assert_eq!(
                output,
                SizingParserOutput {
                    sizing: Some(stoken),
                    errors: Vec::new(),
                }
            );
        }

        #[test]
        fn fixed_ok() {
            let text = "valid_sizing";
            let output = get_sizing(text, 0);
            let data = TokenData::new(text, 0, text.len());
            let stoken = SizingToken { data };
            assert_eq!(
                output,
                SizingParserOutput {
                    sizing: Some(stoken),
                    errors: Vec::new(),
                }
            );
        }
        
        #[test]
        fn dynamic_whitespace_ok() {
            // 2 front spaces, 3 back spaces
            let text = "     ";
            let output = get_sizing(text, 0);
            let length = "".len();
            let stoken = SizingToken{data: TokenData::new(
                &text[5..(length + 5)],
                5,
                length + 5,
            )};
            assert_eq!(
                output,
                SizingParserOutput {
                    sizing: Some(stoken),
                    errors: Vec::new(),
                }
            );
        }
       
        #[test]
        fn dynamic_empty_ok() {
            let text = "";
            let output = get_sizing(text, 0);
            let data = TokenData::new(text, 0, text.len());
            let stoken = SizingToken { data };
            assert_eq!(
                output,
                SizingParserOutput {
                    sizing: Some(stoken),
                    errors: Vec::new(),
                }
            );
        } 


    }

    mod type_spec {
        use super::*;

        #[test]
        fn singleton_whitespace_ok() {
            // 2 front spaces, 3 back spaces
            let text = "  valid_singleton   ";
            let output = get_typespec(text, 0);
            let length = "valid_singleton".len();
            let dtoken = DtypeToken{data: TokenData::new(
                &text[2..(length + 2)],
                2,
                length + 2,
            )};
            assert_eq!(
                output,
                TypeSpecParserOutput {
                    sizing: None,
                    dtype: Some(dtoken),
                    errors: Vec::new(),
                    is_singleton: true,
                }
            );
        }

        #[test]
        fn singleton_ok() {
            let text = "valid_singleton";
            let output = get_typespec(text, 0);
            let data = TokenData::new(text, 0, text.len());
            let dtoken = DtypeToken{data};
            let stoken = None;
            assert_eq!(
                output,
                TypeSpecParserOutput {
                    dtype: Some(dtoken),
                    sizing: stoken,
                    is_singleton: true,
                    errors: Vec::new(),
                }
            );
        }



        #[test]
        fn fixed_whitespace_ok() {
            let whitespace = "  ";
            let dtype_text = "valid_fixed";
            let sizing_text = "10";
            let sizing_body = format!("{whitespace}{sizing_text}{whitespace}");
            let body = format!("{dtype_text}[{sizing_body}]");
            let text = format!("{whitespace}{body}{whitespace}");
            let text = text.as_str();

            let length = dtype_text.len();
            let start = whitespace.len();
            let end = start + length;
            let dtoken = DtypeToken{
                data: TokenData::new(
                    &text[start..(end)],
                    start,
                    end,
            )};
            let length = sizing_text.len();
            let start = whitespace.len() + dtype_text.len() + 1 + whitespace.len();
            let end = start + length;
            let stoken = SizingToken{
                data: TokenData::new(
                    &text[start..(end)],
                    start,
                    end,
            )};
            let output = get_typespec(text, 0);
            assert_eq!(
                output,
                TypeSpecParserOutput {
                    sizing: Some(stoken),
                    dtype: Some(dtoken),
                    errors: Vec::new(),
                    is_singleton: false,
                }
            );
        }

        #[test]
        fn fixed_ok() {
            let whitespace = "";
            let dtype_text = "valid_fixed";
            let sizing_text = "10";
            let sizing_body = format!("{whitespace}{sizing_text}{whitespace}");
            let body = format!("{dtype_text}[{sizing_body}]");
            let text = format!("{whitespace}{body}{whitespace}");
            let text = text.as_str();

            let length = dtype_text.len();
            let start = whitespace.len();
            let end = start + length;
            let dtoken = DtypeToken{
                data: TokenData::new(
                    &text[start..(end)],
                    start,
                    end,
            )};
            let length = sizing_text.len();
            let start = whitespace.len() + dtype_text.len() + 1 + whitespace.len();
            let end = start + length;
            let stoken = SizingToken{
                data: TokenData::new(
                    &text[start..(end)],
                    start,
                    end,
            )};
            let output = get_typespec(text, 0);
            assert_eq!(
                output,
                TypeSpecParserOutput {
                    sizing: Some(stoken),
                    dtype: Some(dtoken),
                    errors: Vec::new(),
                    is_singleton: false,
                }
            );
        }

        #[test]
        fn unexpected_end_of_expression_fails() {
            let whitespace = "";
            let dtype_text = "valid_fixed";
            let sizing_text = "10";
            let sizing_body = format!("{whitespace}{sizing_text}{whitespace}");
            let body = format!("{dtype_text}[{sizing_body}");
            let text = format!("{whitespace}{body}{whitespace}");
            let text = text.as_str();

            let length = dtype_text.len();
            let start = whitespace.len();
            let end = start + length;
            let dtoken = DtypeToken{
                data: TokenData::new(
                    &text[start..(end)],
                    start,
                    end,
            )};
            let output = get_typespec(text, 0);
            assert_eq!(
                output,
                TypeSpecParserOutput {
                    sizing: None,
                    dtype: Some(dtoken),
                    errors: vec![
                        ElucidatorError::Parsing { offender: body, reason: ParsingFailure::UnexpectedEndOfExpression }
                    ],
                    is_singleton: false,
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
            let mut curr_end = ident.chars().count();
            let td_ident = TokenData::new(ident, curr_start, curr_end);
            // colon
            curr_start = curr_end;
            curr_end += 1;
            // starting dtype
            curr_start = curr_end;
            curr_end += dtype.chars().count();
            let td_dtype = TokenData::new(dtype, curr_start, curr_end);
            let t_identifier = IdentifierToken { data: td_ident };
            let t_dtype = DtypeToken { data: td_dtype };
            // Make a string so that the string slice lives to the end of the function
            let size_string;
            let t_sizing = if let Some(size) = sizing {
                // left bracket
                curr_start = curr_end;
                curr_end += 1;
                // starting sizing
                curr_start = curr_end;
                curr_end += size.chars().count();
                size_string = String::from(size);
                Some(SizingToken {
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
            // TODO:
            // We have an inconsistent use of bytes/chars in our codebase
            // This breaks assumptions when we give 2-byte chars in utf8
            // For the moment, we only give it valid ASCII values (1-byte chars)
            const ASCII_MAX: u8 = 127;
            (ASCII_MAX+1..=u8::MAX)
                .map(|x| x as char)
                .filter(|x| x.is_whitespace())
                .collect()
        }

        /// Get random whitespace
        fn get_random_whitespace(num_chars: usize) -> String {
            let whitespace_chars = get_whitespace_chars();
            let result = (0..num_chars)
                .map(|_| random::<usize>() % whitespace_chars.len())
                .map(|x| String::from(whitespace_chars[x]))
                .collect::<Vec<String>>()
                .join("");
            // result
            whitespace_chars.iter().collect::<String>()
        }

        /// Produce a potential whitespace fill
        fn fill() -> String {
            let whitespace = get_random_whitespace(random::<usize>() % 4);
            whitespace
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
            let mut curr_end = lident.chars().count();
            // ident
            curr_start = curr_end;
            curr_end += ident.chars().count();
            let td_ident = TokenData::new(ident, curr_start, curr_end);
            // right ident whitespace
            curr_start = curr_end;
            curr_end += rident.chars().count();
            // colon
            curr_start = curr_end;
            curr_end += 1;
            // left dtype whitespace
            curr_start = curr_end;
            curr_end += ldtype.chars().count();
            // starting dtype
            curr_start = curr_end;
            curr_end += dtype.chars().count();
            let td_dtype = TokenData::new(dtype, curr_start, curr_end);
            let t_identifier = IdentifierToken { data: td_ident };
            let t_dtype = DtypeToken { data: td_dtype };
            // right dtype whitespace
            curr_start = curr_end;
            curr_end += rdtype.chars().count();
            // Make a string so that the string slice lives to the end of the function
            let mut size_string = String::new();
            let t_sizing = if let Some(size) = sizing {
                // left bracket
                curr_start = curr_end;
                curr_end += 1;
                // left sizing whitespace
                curr_start = curr_end;
                curr_end += lsizing.chars().count();
                // starting sizing
                curr_start = curr_end;
                curr_end += size.chars().count();
                size_string = String::from(size);
                if size.chars().count() == 0 {
                    curr_start = curr_end;
                    curr_end += rsizing.chars().count();
                    curr_start = curr_end;
                }
                Some(SizingToken {
                    data: TokenData::new(&size_string, curr_start, curr_end)
                })
            } else { None };

            assert_eq!(output.identifier, Some(t_identifier));
            assert_eq!(output.dtype, Some(t_dtype));
            assert_eq!(output.sizing, t_sizing);
            assert!(output.errors.is_empty());
        }

        #[test]
        fn whitespace_property_singleton_test() {
            for _ in 0..500 {
                run_ok_whitespace("foo", "u8", None);
            }
        }

        #[test]
        fn whitespace_property_dynamic_test() {
            for _ in 0..500 {
                run_ok_whitespace("foo", "u8", Some(""));
            }
        }

        #[test]
        fn whitespace_property_fixed_test() {
            for _ in 0..500 {
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

        #[test]
        fn missing_delimiter_fails() {
            let text = "  foo u8 ";
            let member_spec = get_memberspec(text, 0);
            assert_eq!(
                member_spec,
                MemberSpecParserOutput {
                    identifier: None,
                    dtype: None,
                    sizing: None,
                    errors: vec![
                        ElucidatorError::Parsing {
                            offender: "foo u8".to_string(),
                            reason: ParsingFailure::MissingIdSpecDelimiter
                        }
                    ]
                }
            )
        }
    }

    
}
