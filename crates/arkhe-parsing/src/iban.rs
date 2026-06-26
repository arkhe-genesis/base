use crate::error::{ParseError, ParseResult};
use nom::{IResult, bytes::complete::take_while_m_n, combinator::map, sequence::tuple};

#[derive(Debug, Clone, PartialEq)]
pub struct Iban {
    pub country_code: String,
    pub check_digits: String,
    pub bban: String,
}

impl Iban {
    pub fn parse(input: &str) -> ParseResult<Self> {
        let (_, iban) = parse_iban(input).map_err(|e| ParseError::NomError(e.to_string()))?;
        Ok(iban)
    }

    pub fn formatted(&self) -> String {
        let raw = format!("{}{}{}", self.country_code, self.check_digits, self.bban);
        raw.chars()
            .collect::<Vec<_>>()
            .chunks(4)
            .map(|c| c.iter().collect::<String>())
            .collect::<Vec<_>>()
            .join(" ")
    }
}

fn parse_iban(input: &str) -> IResult<&str, Iban> {
    map(tuple((parse_country_code, parse_check_digits, parse_bban)), |(cc, cd, bban)| Iban {
        country_code: cc.to_string(),
        check_digits: cd.to_string(),
        bban: bban.to_string(),
    })(input)
}

fn parse_country_code(input: &str) -> IResult<&str, &str> {
    take_while_m_n(2, 2, |c: char| c.is_ascii_uppercase())(input)
}

fn parse_check_digits(input: &str) -> IResult<&str, &str> {
    take_while_m_n(2, 2, |c: char| c.is_ascii_digit())(input)
}

fn parse_bban(input: &str) -> IResult<&str, &str> {
    take_while_m_n(1, 30, |c: char| c.is_ascii_alphanumeric())(input)
}
