#[macro_export]
macro_rules! serializable_bitflags {
    ($T:ty) => {
        impl $crate::binary::Deserialize for $T {
            fn deserialize(mut reader: impl ::std::io::Read) -> ::anyhow::Result<Self> {
                use $crate::binary::ReadExt;
                let result = Self::from_bits_retain(reader.deserialize()?);
                Ok(result)
            }
        }
    };
}
