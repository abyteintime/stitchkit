#[macro_export]
macro_rules! serializable_bitflags {
    (type $T:ty; validate |$var:tt| $validate:tt) => {
        impl $crate::binary::Deserialize for $T {
            fn deserialize(
                deserializer: &mut $crate::binary::Deserializer<impl ::std::io::Read>,
            ) -> ::anyhow::Result<Self> {
                let result = Self::from_bits_retain(deserializer.deserialize()?);
                {
                    let $var = result;
                    $validate
                }
                Ok(result)
            }
        }

        impl $crate::binary::Serialize for $T {
            fn serialize(
                &self,
                serializer: &mut $crate::binary::Serializer<impl ::std::io::Write>,
            ) -> ::anyhow::Result<()> {
                $crate::binary::Serialize::serialize(&self.bits(), serializer)
            }
        }
    };
    ($T:ty) => {
        $crate::serializable_bitflags! { type $T; validate |_| {} }
    };
}
