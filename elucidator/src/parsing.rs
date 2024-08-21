use crate::{error::*, token::*};

type Result<T, E = InternalError> = std::result::Result<T, E>;

#[derive(Debug, PartialEq, Clone)]
pub(crate) struct WordParserOutput<'a> {
    word: Option<TokenData<'a>>,
    errors: Vec<InternalError>,
}

#[derive(Debug, PartialEq, Clone)]
pub(crate) struct IdentifierParserOutput<'a> {
    pub identifier: Option<IdentifierToken<'a>>,
    pub errors: Vec<InternalError>,
}

#[derive(Debug, PartialEq, Clone)]
pub(crate) struct DtypeParserOutput<'a> {
    pub dtype: Option<DtypeToken<'a>>,
    pub errors: Vec<InternalError>,
}

#[derive(Debug, PartialEq, Clone)]
pub(crate) struct SizingParserOutput<'a> {
    pub sizing: Option<SizingToken<'a>>,
    pub errors: Vec<InternalError>,
}
#[derive(Debug, PartialEq, Clone)]
pub(crate) struct TypeSpecParserOutput<'a> {
    pub dtype: Option<DtypeToken<'a>>,
    pub sizing: Option<SizingToken<'a>>,
    pub errors: Vec<InternalError>,
    pub is_singleton: bool,
}
#[derive(Debug, PartialEq, Clone)]
pub(crate) struct MemberSpecParserOutput<'a> {
    pub identifier: Option<IdentifierToken<'a>>,
    pub typespec: Option<TypeSpecParserOutput<'a>>,
    pub errors: Vec<InternalError>,
}
#[derive(Debug, PartialEq, Clone)]
pub(crate) struct MetadataSpecParserOutput<'a> {
    pub member_outputs: Vec<MemberSpecParserOutput<'a>>,
    pub errors: Vec<InternalError>,
}

impl<'a> MemberSpecParserOutput<'a> {
    pub fn has_ident(&self) -> bool {
        self.identifier.is_some()
    }

    pub fn has_dtype(&self) -> bool {
        match &self.typespec {
            Some(ts) => {
                ts.dtype.is_some()
            },
            None => { false }
        }
    }

    pub fn has_sizing(&self) -> bool {
        match &self.typespec {
            Some(ts) => {
                ts.sizing.is_some() || ts.is_singleton
            },
            None => { false }
        }
    }
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
        let data_len = data.chars().count();
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
            InternalError::Parsing {
                offender: TokenClone::new(data, start_col),
                reason: ParsingFailure::UnexpectedEndOfExpression,
            }
        );
    };
    if errors.is_empty() {
        let (id_byte_start, _) = id_start.unwrap();
        let trimmed = data.trim();
        let id_byte_end = trimmed.len() + id_byte_start;
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
        let lbracket_byte_pos = data.chars().take(lbracket_pos+1).collect::<String>().len();
        end_of_dtype = data.chars().take(lbracket_pos).collect::<String>().len();
        match contents.chars().position(|c| c == ']') {
            Some(rbracket_pos) => {
                let rbracket_byte_pos = data.chars().take(lbracket_pos + rbracket_pos + 1).collect::<String>().len();
                let byte_start = lbracket_byte_pos;
                let byte_end = rbracket_byte_pos;
                let spo = get_sizing(
                    &data[byte_start..byte_end],
                    start_col + lbracket_pos + 1
                );
                sizing = spo.sizing;
                for error in &spo.errors {
                    errors.push(error.clone());
                }
            },
            None => {
                sizing = None;
                errors.push(
                    InternalError::Parsing {
                        offender: TokenClone::new(
                          &data[lbracket_byte_pos..],
                          start_col + lbracket_pos,
                        ),
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
    let mut typespec = None;
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
        for error in &tso.errors {
            errors.push(error.clone());
        }
        typespec = Some(tso);
    } else {
        let start_non_whitespace = match data.chars().position(|x| !x.is_whitespace()) {
            Some(n) => start_col + n,
            None => start_col,
        };
        errors.push(
            InternalError::Parsing {
                offender: TokenClone::new(
                    data.to_string().trim(), start_non_whitespace
                ),
                reason: ParsingFailure::MissingIdSpecDelimiter
            }
        );
    }

    MemberSpecParserOutput {
        identifier,
        typespec,
        errors,
    }
}

pub fn get_metadataspec<'a>(data: &'a str) -> MetadataSpecParserOutput<'a> {
    let errors: Vec<InternalError>;
    let member_outputs: Vec<MemberSpecParserOutput>; 

    let mut start_positions = data
        .char_indices()
        .filter(|(_, c)| *c == ',')
        .map(|(i, _)| i + 1)
        .collect::<Vec<usize>>();
    start_positions.insert(0, 0);

    if data.chars().all(char::is_whitespace) {
        member_outputs = Vec::new();
    } else if !data.chars().any(|c| c == ',') {
        member_outputs = vec![get_memberspec(data, 0)]
    } else {
        member_outputs = data
            .split(",")
            .zip(start_positions)
            .map(|(member_spec, pos)| get_memberspec(member_spec, pos))
            .collect();
    }

    errors = member_outputs
        .iter()
        .flat_map(|member_output| member_output.errors.iter())
        .map(|e| e.clone())
        .collect();

    MetadataSpecParserOutput {
        member_outputs,
        errors
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::token::TokenClone;
    use rand::random;
    use pretty_assertions::{assert_eq, assert_ne};

    fn lowercase_ascii_chars() -> Vec<char> {
        (u8::MIN..u8::MAX)
            .map(|x| x as char)
            .filter(|x| x.is_ascii_lowercase())
            .collect()
    }

    /// Get the set of whitespace characters
    fn get_whitespace_chars() -> Vec<char> {
        // TODO:
        // We have an inconsistent use of bytes/chars in our codebase
        // This breaks assumptions when we give 2-byte chars in utf8
        // For the moment, we only give it valid ASCII values (1-byte chars)
        (u8::MIN..=u8::MAX)
            .map(|x| x as char)
            .filter(|x| x.is_whitespace())
            .collect()
    }

    fn random_lowercase_ascii_char() -> char {
        lowercase_ascii_chars()[
            random::<usize>() % lowercase_ascii_chars().len()
        ]
    }

    fn random_word() -> String {
        let id_len = (random::<u8>() % 9) + 1;
        (0..id_len)
            .map(|_| random_lowercase_ascii_char())
            .collect()
    }

    fn random_sizing(whitespace: bool) -> String {
        match random::<u8>() % 3 {
            0 => {
                // Singleton
                String::from("")
            },
            1 => {
                // Fixed
                let size: u8 = random();
                if whitespace {
                    format!("[{}{size}{}]", fill(), fill())
                } else {
                    format!("[{size}]")
                }
            },
            2 => {
                // Dynamic
                if whitespace {
                    format!("[{}]", fill())
                } else {
                    String::from("[]")
                }
            },
            _ => { unreachable!(); },
        }
    }

    fn random_memberspec() -> String {
        let identifier = random_word();
        let dtype = random_word();
        let sizing = random_sizing(false);
        format!("{identifier}:{dtype}{sizing}")
    }

    fn random_memberspec_whitespace() -> String {
        let lidentifier = fill();
        let identifier = random_word();
        let ridentifier = fill();
        let ldtype = fill();
        let dtype = random_word();
        let rdtype = fill();
        let sizing = random_sizing(true);
        let endspace = fill();
        let idblock = format!("{lidentifier}{identifier}{ridentifier}");
        let dtypeblock = format!("{ldtype}{dtype}{rdtype}");
        format!("{idblock}:{dtypeblock}{sizing}{endspace}")
    }

    /// Get random whitespace
    fn random_whitespace(num_chars: usize) -> String {
        let whitespace_chars = get_whitespace_chars();
        (0..num_chars)
            .map(|_| random::<usize>() % whitespace_chars.len())
            .map(|x| String::from(whitespace_chars[x]))
            .collect::<Vec<String>>()
            .join("")
    }

    /// Produce a potential whitespace fill
    fn fill() -> String {
        let whitespace = random_whitespace(random::<usize>() % 4);
        whitespace
    }

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
            pretty_assertions::assert_eq!(
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
            pretty_assertions::assert_eq!(
                output,
                WordParserOutput {
                    word: Some(data),
                    errors: Vec::new(),
                }
            );
        }

        #[test]
        fn only_whitespace_err() {
            let text = "    ";
            let output = get_word(text, 0);
            pretty_assertions::assert_eq!(
                output,
                WordParserOutput{
                    word: None,
                    errors: vec![
                        InternalError::Parsing {
                            offender: TokenClone::new(text, 0),
                            reason: ParsingFailure::UnexpectedEndOfExpression
                        }
                    ],
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
            pretty_assertions::assert_eq!(
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
            pretty_assertions::assert_eq!(
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
            pretty_assertions::assert_eq!(
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
            pretty_assertions::assert_eq!(
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
            pretty_assertions::assert_eq!(
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
            pretty_assertions::assert_eq!(
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
            pretty_assertions::assert_eq!(
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
            pretty_assertions::assert_eq!(
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
            pretty_assertions::assert_eq!(
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
            pretty_assertions::assert_eq!(
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
            pretty_assertions::assert_eq!(
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
            pretty_assertions::assert_eq!(
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
            pretty_assertions::assert_eq!(
                output,
                TypeSpecParserOutput {
                    sizing: None,
                    dtype: Some(dtoken),
                    errors: vec![
                        InternalError::Parsing {
                            offender: TokenClone::new(&sizing_body, 11),
                            reason: ParsingFailure::UnexpectedEndOfExpression }
                    ],
                    is_singleton: false,
                }
            );
        }
    }

    // Tests marked "invalid" are invalid according to the standard, but are parseable.
    mod member_spec {
        use super::*;
    
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

            pretty_assertions::assert_eq!(output.identifier, Some(t_identifier));
            pretty_assertions::assert_eq!(output.typespec.clone().unwrap().dtype, Some(t_dtype));
            pretty_assertions::assert_eq!(output.typespec.clone().unwrap().sizing, t_sizing);
            assert!(output.errors.is_empty());
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

            pretty_assertions::assert_eq!(output.identifier, Some(t_identifier));
            pretty_assertions::assert_eq!(output.typespec.clone().unwrap().dtype, Some(t_dtype));
            pretty_assertions::assert_eq!(output.typespec.clone().unwrap().sizing, t_sizing);
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
        fn ok_invalid_space_in_ident() {
            run_ok_simple("foo bar", "u8", None)
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
            pretty_assertions::assert_eq!(
                member_spec,
                MemberSpecParserOutput {
                    identifier: None,
                    typespec: None,
                    errors: vec![
                        InternalError::Parsing {
                            offender: TokenClone::new("foo u8", 2),
                            reason: ParsingFailure::MissingIdSpecDelimiter
                        }
                    ]
                }
            )
        }
    }

    mod metadata {
        use super::*;

        fn run_property_test(whitespace: bool) {
            let n_specs = random::<u8>() % 10 + 1;
            let generator = if whitespace {
                random_memberspec_whitespace
            } else {
                random_memberspec
            };
            let member_specs: Vec<String> = (0..n_specs)
                .map(|_| generator())
                .collect();
            let metadata_spec_text = member_specs.join(",");
            let mut start_positions = metadata_spec_text
                .char_indices()
                .filter(|(_, c)| *c == ',')
                .map(|(i, _)| i + 1)
                .collect::<Vec<usize>>();
            start_positions.insert(0, 0);
            let parsed_members: Vec<MemberSpecParserOutput> = member_specs
                .iter()
                .zip(start_positions.iter())
                .map(|(x, pos)| get_memberspec(x, *pos))
                .collect();
            pretty_assertions::assert_eq!(start_positions.len(), parsed_members.len());
            let metadata_spec = get_metadataspec(&metadata_spec_text);
            pretty_assertions::assert_eq!(
                metadata_spec,
                MetadataSpecParserOutput {
                    member_outputs: parsed_members,
                    errors: Vec::new(),
                }
            );
        }

        #[test]
        fn no_comma_ok() {
            let spec = "foo:u8";
            let metadata_spec = get_metadataspec(spec);
            pretty_assertions::assert_eq!(
                metadata_spec,
                MetadataSpecParserOutput {
                    member_outputs: vec![
                        get_memberspec(spec, 0),
                    ],
                    errors: Vec::new(),
                }
            );
        }

        #[test]
        fn two_members_no_whitespace() {
            let m1 = "foo:u8[10]";
            let m2 = "bar:i32[]";
            let spec = &format!("{m1},{m2}");
            let metadata_spec = get_metadataspec(spec);
            pretty_assertions::assert_eq!(
                metadata_spec,
                MetadataSpecParserOutput {
                    member_outputs: vec![
                        get_memberspec(m1, 0),
                        get_memberspec(m2, m1.chars().count() + 1),
                    ],
                    errors: Vec::new(),
                }
            );
        }

        #[test]
        fn no_whitespace_property_ok() {
            for _ in 0..500 {
                run_property_test(false)
            }
        }

        #[test]
        fn whitespace_property_ok() {
            for _ in 0..500 {
                run_property_test(true)
            }
        }

        #[test]
        fn blank_spec_ok() {
            let spec = "";
            let metadata_spec = get_metadataspec(spec);
            pretty_assertions::assert_eq!(
                metadata_spec,
                MetadataSpecParserOutput {
                    member_outputs: Vec::new(),
                    errors: Vec::new(),
                },
            );
        }

        // TODO: handle case where some memberspecs are erroneous and others aren't
        #[test]
        fn some_ok_some_not() {
            let member_specs = [
                "woofs: u8",
                ": f32[",
                "splashes: i32[100]",
                "flaps: []",
                "meows: i32",
            ];
            let spec = member_specs.join(",");
            let mut start_positions = spec
                .char_indices()
                .filter(|(_, c)| *c == ',')
                .map(|(i, _)| i + 1)
                .collect::<Vec<usize>>();
            start_positions.insert(0, 0);
            let parsed_members: Vec<MemberSpecParserOutput> = member_specs
                .iter()
                .zip(start_positions.iter())
                .map(|(x, pos)| get_memberspec(x, *pos))
                .collect();
            pretty_assertions::assert_eq!(start_positions.len(), parsed_members.len());
            let metadata_spec = get_metadataspec(&spec);
            let expected_errors: Vec<InternalError> = parsed_members
                .iter()
                .flat_map(|x| x.errors.iter())
                .map(|x| x.clone())
                .collect();
            pretty_assertions::assert_eq!(
                metadata_spec,
                MetadataSpecParserOutput {
                    member_outputs: parsed_members,
                    errors: expected_errors,
                }
            );
        }

        // TODO: handle case where all memberspecs are invalid
    }

    
}
