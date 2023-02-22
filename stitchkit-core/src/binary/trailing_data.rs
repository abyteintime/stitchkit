use std::{io::Read, ops::Deref};

use anyhow::Context;

use super::{Deserialize, Deserializer};

#[derive(Debug, Clone)]
pub struct TrailingData(pub Vec<u8>);

impl Deserialize for TrailingData {
    fn deserialize(deserializer: &mut Deserializer<impl Read>) -> anyhow::Result<Self> {
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
