pub(crate) struct TokenData<'a> {
    data: &'a str,
    line_start: usize,
    line_end: usize,
    column_start: usize,
    column_end: usize, 
} 

pub(crate) struct IdentifierToken<'a> {
    data: TokenData<'a>,
}

pub(crate) struct DtypeToken<'a> {
    data: TokenData<'a>,
}

pub(crate) struct SizeToken<'a> {
    data: TokenData<'a>,
}

pub(crate) struct DelimiterToken<'a> {
    data: TokenData<'a>,
}

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
    fn get_line_start(&self) -> usize {
        match &self {
            Token::Identifier(token) =>  token.data.line_start,
            Token::Dtype(token) =>  token.data.line_start,
            Token::Size(token) =>  token.data.line_start,
            Token::Delimiter(token) =>  token.data.line_start,
        }
    }
    fn get_line_end(&self) -> usize {
        match &self {
            Token::Identifier(token) =>  token.data.line_end,
            Token::Dtype(token) =>  token.data.line_end,
            Token::Size(token) =>  token.data.line_end,
            Token::Delimiter(token) =>  token.data.line_end,
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