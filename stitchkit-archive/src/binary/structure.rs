#[macro_export]
macro_rules! serializable_structure {
    (
        type $T:ty { $($fields:tt),+ $(,)? }
        $(deserialize_extra |$var:tt| $code:tt)?
    ) => {
        impl $crate::binary::Deserialize for $T {
            fn deserialize(mut reader: impl ::std::io::Read) -> ::anyhow::Result<Self> {
                use ::anyhow::Context;
                let result = Self {
                    $($fields: reader.deserialize().context(concat!("cannot deserialize field ", stringify!($T), "::", stringify!($fields)))?),*
                };
                $(let $var = &result; $code)?
                Ok(result)
            }
        }
    };
}
