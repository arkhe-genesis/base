use nom::{
    IResult,
    bytes::complete::{tag, take_while_m_n},
    combinator::map,
    sequence::tuple,
};

use crate::error::{ParseError, ParseResult};

#[derive(Debug, Clone, PartialEq)]
pub struct Did {
    pub method: String,
    pub identifier: String,
}

impl Did {
    pub fn parse(input: &str) -> ParseResult<Self> {
        let (_, did) = parse_did(input).map_err(|e| ParseError::NomError(e.to_string()))?;
        Ok(did)
    }

    pub fn to_string(&self) -> String {
        format!("did:{}:{}", self.method, self.identifier)
    }
}

fn parse_did(input: &str) -> IResult<&str, Did> {
    map(
        tuple((tag("did:"), parse_method, tag(":"), parse_identifier)),
        |(_, method, _, identifier)| Did {
            method: method.to_string(),
            identifier: identifier.to_string(),
        },
    )(input)
}

fn parse_method(input: &str) -> IResult<&str, &str> {
    take_while_m_n(1, 20, |c: char| c.is_ascii_lowercase())(input)
}

fn parse_identifier(input: &str) -> IResult<&str, &str> {
    take_while_m_n(43, 44, |c: char| c.is_ascii_alphanumeric())(input)
}
