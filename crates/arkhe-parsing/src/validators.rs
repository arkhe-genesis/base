use crate::error::{ParseError, ParseResult};
use crate::regex::SafeRegex;

pub fn validate_did(did: &str) -> ParseResult<()> {
    let re = SafeRegex::new(r"^did:arkhe:[a-zA-Z0-9]{43,44}$")?;
    if !re.is_match(did)? {
        return Err(ParseError::InvalidFormat("DID must match did:arkhe:<43-44 base58 chars>".into()));
    }
    if has_invisible_unicode(did) {
        return Err(ParseError::SuspiciousCharacters("DID contains invisible or control characters".into()));
    }
    if did.contains('\0') {
        return Err(ParseError::SuspiciousCharacters("DID contains null byte".into()));
    }
    let parts: Vec<&str> = did.split(':').collect();
    if parts.len() != 3 || parts[0] != "did" || parts[1] != "arkhe" {
        return Err(ParseError::InvalidFormat("DID must have exactly 3 parts: did:arkhe:<id>".into()));
    }
    Ok(())
}

pub fn validate_iban(iban: &str) -> ParseResult<()> {
    let cleaned: String = iban.chars().filter(|c| !c.is_whitespace()).collect();
    let re = SafeRegex::new(r"^[A-Z]{2}[0-9]{2}[A-Z0-9]{1,30}$")?;
    if !re.is_match(&cleaned)? {
        return Err(ParseError::InvalidFormat("IBAN must match [A-Z]{2}[0-9]{2}[A-Z0-9]{1,30}".into()));
    }
    if cleaned.len() < 5 {
        return Err(ParseError::InvalidLength { expected: 5, actual: cleaned.len() });
    }
    if !mod97_check(&cleaned) {
        return Err(ParseError::InvalidChecksum { expected: "1 (MOD-97)".into(), actual: "invalid".into() });
    }
    Ok(())
}

pub fn validate_bic(bic: &str) -> ParseResult<()> {
    let re = SafeRegex::new(r"^[A-Z]{4}[A-Z]{2}[A-Z0-9]{2}([A-Z0-9]{3})?$")?;
    if !re.is_match(bic)? {
        return Err(ParseError::InvalidFormat("BIC must be 8 or 11 characters: BBBBCCLL[BBB]".into()));
    }
    if bic.len() != 8 && bic.len() != 11 {
        return Err(ParseError::InvalidLength { expected: 8, actual: bic.len() });
    }
    Ok(())
}

fn has_invisible_unicode(s: &str) -> bool {
    s.chars().any(|c| {
        c == '\u{200B}' || c == '\u{200C}' || c == '\u{200D}' || c == '\u{FEFF}' || c == '\u{2060}'
            || (c.is_control() && c != '\t' && c != '\n' && c != '\r')
    })
}

fn mod97_check(iban: &str) -> bool {
    let rearranged = format!("{}{}", &iban[4..], &iban[..4]);
    let mut numeric = String::new();
    for c in rearranged.chars() {
        if c.is_ascii_alphabetic() {
            let val = (c.to_ascii_uppercase() as u8 - b'A' + 10) as u32;
            numeric.push_str(&val.to_string());
        } else {
            numeric.push(c);
        }
    }
    let mut remainder: u32 = 0;
    for c in numeric.chars() {
        let digit = c.to_digit(10).unwrap();
        remainder = (remainder * 10 + digit) % 97;
    }
    remainder == 1
}
