use std::{fmt::Display, str::FromStr};

use no_panic::no_panic;

use super::either::Either;

#[derive(Debug, Eq, PartialEq)]
pub struct RadioIdentifier {
    pub org: String,
    pub county: String,
    pub agency: u8,
    pub engine_type: u32,
    pub number: u32,
}

impl RadioIdentifier {
    pub fn to_left<B>(self) -> Either<RadioIdentifier, B> {
        return Either::Left(self);
    }
}

impl FromStr for RadioIdentifier {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut in_stream = s.chars();

        let org: String = in_stream.by_ref().take_while(|c| c != &' ').collect();
        let county: String = in_stream.by_ref().take_while(|c| c != &' ').collect();

        let agency = in_stream
            .by_ref() // use a reference to actually advance the common iterator and not create a copy
            .take_while(|c| c != &'/')
            .collect::<String>()
            .parse::<u8>()
            .map_err(|e| format!("Failed to parse RadioIdentifier {} at Agency: {}", s, e))?;

        let engine_type = in_stream
            .by_ref()
            .take_while(|c| c != &'-')
            .collect::<String>()
            .parse::<u32>()
            .map_err(|e| {
                format!(
                    "Failed to parse RadioIdentifier {} at engine type: {}",
                    s, e
                )
            })?;

        let number = in_stream
            .by_ref()
            .take_while(|c| c.is_numeric())
            .collect::<String>()
            .parse::<u32>()
            .map_err(|e| {
                format!(
                    "Failed to parse RadioIdentifier {} at engine number: {}",
                    s, e
                )
            })?;

        if in_stream.any(|c| !c.is_whitespace()) {
            return Err(format!(
                "Failed to parse RadioIdentifier {} remaining characters at end of string",
                s
            ));
        }

        return Ok(RadioIdentifier {
            org: org,
            county: county,
            agency: agency,
            engine_type: engine_type,
            number: number,
        });
    }
}

impl Display for RadioIdentifier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} {} {:02}/{:02}-{:02}",
            self.org, self.county, self.agency, self.engine_type, self.number
        )
    }
}

#[test]
fn test_parse_valid_radio_identifier() {
    const VALID_RADIO_IDENTIFIER: &str = "FL BRB 01/16-21";

    let radio_identifier = RadioIdentifier::from_str(VALID_RADIO_IDENTIFIER).unwrap();

    assert_eq!(radio_identifier.org, "FL");
    assert_eq!(radio_identifier.county, "BRB");
    assert_eq!(radio_identifier.agency, 1);
    assert_eq!(radio_identifier.engine_type, 16);
    assert_eq!(radio_identifier.number, 21);
}

#[test]
fn test_parse_invalid_radio_identifier() {
    const INVALID_RADIO_IDENTIFIER1: &str = "FL BRB AA/BB-ALPHA";
    const INVALID_RADIO_IDENTIFIER2: &str = "FL BRB 01/BB-AA";
    const INVALID_RADIO_IDENTIFIER3: &str = "FL BRB 01/16-AA";
    const INVALID_RADIO_IDENTIFIER4: &str = "FL BRB 01/16-21-AA"; // excess garbage
    const INVALID_RADIO_IDENTIFIER5: &str = "FL BRB 01-01-01"; // wrong format for mails, still common in the wild.

    let radio_identifier1 = RadioIdentifier::from_str(INVALID_RADIO_IDENTIFIER1);
    let radio_identifier2 = RadioIdentifier::from_str(INVALID_RADIO_IDENTIFIER2);
    let radio_identifier3 = RadioIdentifier::from_str(INVALID_RADIO_IDENTIFIER3);
    let radio_identifier4 = RadioIdentifier::from_str(INVALID_RADIO_IDENTIFIER4);
    let radio_identifier5 = RadioIdentifier::from_str(INVALID_RADIO_IDENTIFIER5);

    assert!(radio_identifier1.is_err());
    assert!(radio_identifier2.is_err());
    assert!(radio_identifier3.is_err());
    assert!(radio_identifier4.is_err());
    assert!(radio_identifier5.is_err());
}

#[test]
fn test_display_radio_identifier() {
    const VALID_RADIO_IDENTIFIER: &str = "FL BRB 01/16-21";

    let radio_identifier = RadioIdentifier::from_str(VALID_RADIO_IDENTIFIER).unwrap();
    assert_eq!(format!("{}", radio_identifier), VALID_RADIO_IDENTIFIER);
}
