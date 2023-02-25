#[macro_export]
macro_rules! serializable_bitflags {
    (type $T:ty; validate |$var:tt| $validate:tt) => {
        impl $crate::binary::Deserialize for $T {
            fn deserialize(
                deserializer: &mut $crate::binary::Deserializer<impl ::std::io::Read>,
            ) -> ::std::result::Result<Self, $crate::binary::Error> {
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
            ) -> ::std::result::Result<(), $crate::binary::Error> {
                $crate::binary::Serialize::serialize(&self.bits(), serializer)
            }
        }
    };
    ($T:ty) => {
        $crate::serializable_bitflags! { type $T; validate |_| {} }
    };
}
