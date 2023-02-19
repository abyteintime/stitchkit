use std::{io::Read, ops::Deref};

use anyhow::Context;

use super::{Deserialize, Deserializer};

impl<T> Deserialize for Vec<T>
where
    T: Deserialize,
{
    fn deserialize(deserializer: &mut Deserializer<impl Read>) -> anyhow::Result<Self> {
        let len = deserializer
            .deserialize::<u32>()
            .context("cannot read array length")? as usize;
        let mut vec = Vec::with_capacity(len);
        for i in 0..len {
            vec.push(deserializer.deserialize().with_context(|| {
                format!("cannot deserialize array field {i} (array of length {len})")
            })?);
        }
        Ok(vec)
    }
}

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
