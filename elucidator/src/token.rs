/// Store data about a token, including the relevant string slice and its character span, with the
/// end location being exclusive.
/// ```ignore
/// use elucidator::token::TokenData;
///
/// let big_slice = "cat: i32, dog: i32";
///
/// // extract the token for cat
/// let td_cat = TokenData::new(big_slice[0..3], 0, 3);
/// // extract the token for dog
/// let td_dog = TokenData::new(big_slice[10..13], 10, 13);
/// # assert_eq!(td_cat.data, "cat");
/// # assert_eq!(td_dog.data, "dog");
/// ```
#[derive(Debug, PartialEq)]
pub(crate) struct TokenData<'a> {
    data: &'a str,
    column_start: usize,
    column_end: usize, 
}

impl<'a> TokenData<'a> {
    pub fn new(data: &'a str, column_start: usize, column_end: usize) -> TokenData {
        assert!(column_start <= column_end, "columns swapped");
        assert!(data.len() == column_end - column_start, "bad sizing");
        TokenData { data, column_start, column_end }
    }
}

#[derive(Debug, PartialEq)]
pub(crate) struct IdentifierToken<'a> {
    pub data: TokenData<'a>,
}

#[derive(Debug, PartialEq)]
pub(crate) struct DtypeToken<'a> {
    pub data: TokenData<'a>,
}

#[derive(Debug, PartialEq)]
pub(crate) struct SizeToken<'a> {
    pub data: TokenData<'a>,
}

#[derive(Debug, PartialEq)]
pub(crate) struct DelimiterToken<'a> {
    pub data: TokenData<'a>,
}

#[derive(Debug, PartialEq)]
pub(crate) enum Token<'a> {
    Identifier(IdentifierToken<'a>),
    Dtype(DtypeToken<'a>),
    Size(SizeToken<'a>),
    Delimiter(DelimiterToken<'a>),
}

impl <'a> Token<'a> {
    fn get_data(&self) -> &str {
        match &self {
            Token::Identifier(token) =>  token.data.data,
            Token::Dtype(token) =>  token.data.data,
            Token::Size(token) =>  token.data.data,
            Token::Delimiter(token) =>  token.data.data,
        }
    }
    fn get_column_start(&self) -> usize {
        match &self {
            Token::Identifier(token) =>  token.data.column_start,
            Token::Dtype(token) =>  token.data.column_start,
            Token::Size(token) =>  token.data.column_start,
            Token::Delimiter(token) =>  token.data.column_start,
        }
    }
    fn get_column_end(&self) -> usize {
        match &self {
            Token::Identifier(token) =>  token.data.column_end,
            Token::Dtype(token) =>  token.data.column_end,
            Token::Size(token) =>  token.data.column_end,
            Token::Delimiter(token) =>  token.data.column_end,
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    #[should_panic(expected = "bad sizing")]
    fn token_data_new_bad_span_size_err() {
        let _ = TokenData::new("cat", 0, 1);
    }

    #[test]
    #[should_panic(expected = "columns swapped")]
    fn token_data_new_swapped_cols_err() {
        let _ = TokenData::new("cat", 2, 0);
    }

    #[test]
    fn token_data_new_empty_ok() {
        let _ = TokenData::new("", 0, 0);
    }

    #[test]
    fn token_data_new_one_char_ok() {
        let _ = TokenData::new("c", 0, 1);
    }

    #[test]
    fn token_data_new_ok() {
        let _ = TokenData::new("cat", 0, 3);
    }

}
