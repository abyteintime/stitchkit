#[macro_export]
macro_rules! serializable_bitflags {
    (type $T:ty; validate |$var:tt| $validate:tt) => {
        impl $crate::binary::Deserialize for $T {
            fn deserialize(mut reader: impl ::std::io::Read) -> ::anyhow::Result<Self> {
                use $crate::binary::ReadExt;
                let result = Self::from_bits_retain(reader.deserialize()?);
                {
                    let $var = result;
                    $validate
                }
                Ok(result)
            }
        }
    };
    ($T:ty) => {
        $crate::serializable_bitflags! { type $T; validate |_| {} }
    };
}
