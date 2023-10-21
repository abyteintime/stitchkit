use bitflags::bitflags;

use crate::serializable_bitflags;

bitflags! {
    /// `UObject` flags.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
    pub struct ObjectFlags: u64 {
        const DEFAULT        = 0x0000000000000200;
        const TRANSACTIONAL  = 0x0000000100000000;
        const PUBLIC         = 0x0000000400000000;
        const TRANSIENT      = 0x0000400000000000;
        // These three need to be figured out because they seem kind of important.
        const UNKNOWN_1      = 0x0001000000000000;
        const UNKNOWN_2      = 0x0002000000000000;
        const UNKNOWN_3      = 0x0004000000000000;
        const STANDALONE     = 0x0008000000000000;
        const NOT_FOR_CLIENT = 0x0010000000000000;
        const NOT_FOR_SERVER = 0x0020000000000000;
        const NOT_FOR_EDIT   = 0x0040000000000000;

        /// These flags are present on all names in the name table.
        const NAME           = 0x0007001000000000;
    }
}

serializable_bitflags!(ObjectFlags);
