use std::fmt::{Debug, Display};

use anyhow::Context;

use crate::binary::{Deserialize, ReadExt};

#[derive(Clone, PartialEq, Eq, Default, Hash)]
pub struct UnrealString {
    bytes: Vec<u8>,
}

impl Deserialize for UnrealString {
    fn deserialize(mut reader: impl std::io::Read) -> anyhow::Result<Self> {
        let length = reader.deserialize::<u32>()?;
        let mut bytes = vec![0; length as usize];
        reader
            .read_exact(&mut bytes)
            .with_context(|| format!("cannot read string of length {length}"))?;
        Ok(Self { bytes })
    }
}

impl Debug for UnrealString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Ok(utf8) = std::str::from_utf8(&self.bytes) {
            if let Some(nul_terminated) = utf8.strip_suffix('\0') {
                Debug::fmt(nul_terminated, f)?;
                f.write_str(" <NUL>")
            } else {
                Debug::fmt(utf8, f)
            }
        } else {
            f.write_str("<invalid UTF-8> ")?;
            Debug::fmt(&self.bytes, f)
        }
    }
}

impl Display for UnrealString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Ok(utf8) = std::str::from_utf8(&self.bytes) {
            if let Some(nul_terminated) = utf8.strip_suffix('\0') {
                Display::fmt(nul_terminated, f)
            } else {
                Display::fmt(utf8, f)
            }
        } else {
            f.write_str("<invalid UTF-8> ")?;
            Debug::fmt(&self.bytes, f)
        }
    }
}
