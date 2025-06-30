use std::{borrow::Borrow, str::FromStr, sync::Arc};

use crate::util::AsPartial;

#[derive(thiserror::Error, Debug, Clone)]
pub enum OrcidParsingError{
    #[error("Bad ORCID string")]
    BadCode,
    #[error("Bad checksum")]
    BadChecksum,
    #[error("Bad ORCID char: {0}")]
    BadChar(char),
    #[error("Bad ORCID checksum char: {0}")]
    BadChecksumChar(char),
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(into = "String")]
#[serde(try_from = "String")]
pub struct Orcid{
    digits: Arc<str>,
    value: u64,
    checksum: u64,
}

impl AsPartial for Orcid{
    type Partial = String;
}

impl Borrow<str> for Orcid{
    fn borrow(&self) -> &str {
        &self.digits
    }
}

trait CharExt{
    fn try_as_orcid_digit(self) -> Result<u64, OrcidParsingError>;
    fn try_as_orcid_checksum(self) -> Result<u64, OrcidParsingError>;
}
impl CharExt for char{
    fn try_as_orcid_digit(self) -> Result<u64, OrcidParsingError>{
        self.to_digit(10).ok_or(OrcidParsingError::BadChar(self)).map(|x| x as u64)
    }
    fn try_as_orcid_checksum(self) -> Result<u64, OrcidParsingError>{
        if self == 'X'{
            return Ok(10)
        }
        return self.try_as_orcid_digit()
    }
}

impl From<Orcid> for String{
    fn from(value: Orcid) -> String {
        value.digits.as_ref().to_owned()
    }
}

impl FromStr for Orcid{
    type Err = OrcidParsingError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        TryFrom::<&str>::try_from(s)
    }
    
}

impl TryFrom<&str> for Orcid{
    type Error = OrcidParsingError;
    fn try_from(value: &str) -> Result<Self, Self::Error>{
        let parts = value.split("-")
            .map(|part_str| part_str.chars().collect::<Vec<_>>())
            .map(|part_vec| {
                TryInto::<[char; 4]>::try_into(part_vec)
                    .map_err(|_| OrcidParsingError::BadCode)
            })
            .collect::<Result<Vec<[char; 4]>, _>>()?;
        let Ok(four_parts) = TryInto::<[[char; 4]; 4]>::try_into(parts) else {
            return Err(OrcidParsingError::BadCode);
        };

        let chars = four_parts[0].iter()
            .chain(four_parts[1].iter())
            .chain(four_parts[2].iter())
            .chain(four_parts[3][0..3].iter());

        let mut checksum_total: u64 = 0;
        let orcid_value = chars.enumerate().try_fold(0u64, |acc, (idx, ch)|{
            let digit = ch.try_as_orcid_digit()?;
            checksum_total = (checksum_total + digit) * 2;
            let digit_value = digit * 10u64.pow(15 - 1 - idx as u32);
            Ok(acc +  digit_value)
        })?;

        let checksum_remainder = checksum_total % 11;
        let expected_checksum = (12 - checksum_remainder) % 11;
        let found_checksum = four_parts[3][3].try_as_orcid_checksum()?;

        if expected_checksum != found_checksum{
            return Err(OrcidParsingError::BadChecksum)
        }
        Ok(Self{digits: Arc::from(value), value: orcid_value, checksum: found_checksum})
    }
}

impl TryFrom<String> for Orcid{
    type Error = OrcidParsingError;
    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::try_from(value.as_str())
    }
}

#[test]
fn test_orcid_parsing(){
    let good_raw_orcid: String = "0000-0001-7051-1197".into();
    let reproduced_orcid: String = Orcid::try_from(good_raw_orcid.clone()).unwrap().into();
    assert_eq!(good_raw_orcid, reproduced_orcid);

    let good_raw_orcid: String = "0000-0002-8205-121X".into();
    let reproduced_orcid: String = Orcid::try_from(good_raw_orcid.clone()).unwrap().into();
    assert_eq!(good_raw_orcid, reproduced_orcid);

    let bad_raw_orcid: String = "0000-0001-7051-119X".into();
    assert!(Orcid::try_from(bad_raw_orcid).is_err());
}
