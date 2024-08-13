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
pub(crate) struct SizingToken<'a> {
    pub data: TokenData<'a>,
}

#[derive(Debug, PartialEq)]
pub(crate) struct DelimiterToken<'a> {
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

}
