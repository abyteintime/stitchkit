use std::{io::Read, ops::Deref};

use super::{Deserialize, Deserializer, Error, ResultContextExt};

#[derive(Debug, Clone)]
pub struct TrailingData(pub Vec<u8>);

impl Deserialize for TrailingData {
    fn deserialize(deserializer: &mut Deserializer<impl Read>) -> Result<Self, Error> {
        let mut buffer = vec![];
        deserializer
            .read_to_end(&mut buffer)
            .context("cannot deserialize trailing data")?;
        Ok(Self(buffer))
    }
}

impl Deref for TrailingData {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
