use std::fmt;

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
#[derive(Debug, PartialEq, Clone)]
pub(crate) struct TokenData<'a> {
    pub data: &'a str,
    pub column_start: usize,
    pub column_end: usize,
}

impl<'a> TokenData<'a> {
    pub fn new(data: &'a str, column_start: usize, column_end: usize) -> TokenData {
        let column_width = data.chars().count();
        assert!(column_start <= column_end, "columns swapped");
        assert!(
            data.chars().count() == column_end - column_start,
            "Bad sizing; start {column_start} end {column_end}, column width {column_width}, slice {data}"
        );
        TokenData {
            data,
            column_start,
            column_end,
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub(crate) struct TokenClone {
    pub data: String,
    pub column_start: usize,
    pub column_end: usize,
}

impl TokenClone {
    pub fn new(data: &str, column_start: usize) -> Self {
        let column_end = column_start + data.chars().count();
        let td = TokenData::new(data, column_start, column_end);
        Self::from_token_data(&td)
    }
    pub fn from_token_data(token: &TokenData) -> Self {
        TokenClone {
            data: token.data.to_string(),
            column_start: token.column_start,
            column_end: token.column_end,
        }
    }
}

impl fmt::Display for TokenClone {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Cols {}-{}: {}",
            self.column_start, self.column_end, self.data
        )
    }
}

#[derive(Debug, PartialEq, Clone)]
pub(crate) struct IdentifierToken<'a> {
    pub data: TokenData<'a>,
}

#[derive(Debug, PartialEq, Clone)]
pub(crate) struct DtypeToken<'a> {
    pub data: TokenData<'a>,
}

#[derive(Debug, PartialEq, Clone)]
pub(crate) struct SizingToken<'a> {
    pub data: TokenData<'a>,
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    #[should_panic(expected = "Bad sizing")]
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

    #[test]
    fn token_data_to_clone_ok() {
        let td = TokenData::new("cat", 0, 3);
        let tc = TokenClone::from_token_data(&td);
        assert_eq!(
            tc,
            TokenClone {
                data: "cat".to_string(),
                column_start: 0,
                column_end: 3,
            }
        );
    }
}
