use crate::{error::*, member::MemberSpecification, token::*};

type Result<T, E = ElucidatorError> = std::result::Result<T, E>;

pub(crate) struct IdentifierParserOutput<'a> {
    token: Option<IdentifierToken<'a>>,
}

impl <'a> IdentifierParser<'a> {
    fn get_token(&self) -> Token<'a> {
        Ok(Token::Identifier(self.token.unwrap()))
    }
}
