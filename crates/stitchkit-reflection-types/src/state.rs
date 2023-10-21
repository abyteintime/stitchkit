use bitflags::bitflags;
use stitchkit_archive::{index::OptionalPackageObjectIndex, name::ArchivedName};
use stitchkit_core::{primitive::ConstI16, serializable_bitflags, Deserialize, Serialize};

use crate::Chunk;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct State {
    pub chunk: Chunk<()>,
    /// Events implemented by this state. For an event to count as implemented, its body must
    /// not be empty.
    pub implements_events: Events,
    /// Purpose unknown; it's probed by `GotoState` but all game classes have this set to -1.
    pub _unknown: ConstI16<-1>,
    /// Contains a bitmask of events that are enabled whenever the state is entered.
    pub enables_events: Events,
    /// Functions declared in this state.
    pub function_map: Vec<FunctionMapEntry>,
}

bitflags! {
    /// Events that can be `Enabled`d and `Disable`d.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
    pub struct Events: u32 {
        const DESTROYED         = 0x00000001;
        const GAINED_CHILD      = 0x00000002;
        const LOST_CHILD        = 0x00000004;
        const HIT_WALL          = 0x00000008;
        const FALLING           = 0x00000010;
        const LANDED            = 0x00000020;
        const TOUCH             = 0x00000040;
        const UNTOUCH           = 0x00000080;
        const BUMP              = 0x00000100;
        const BEGIN_STATE       = 0x00000200;
        const END_STATE         = 0x00000400;
        const BASE_CHANGE       = 0x00000800;
        const ATTACH            = 0x00001000;
        const DETACH            = 0x00002000;
        const ENCROACHING_ON    = 0x00004000;
        const ENCROACHED_BY     = 0x00008000;
        const MAY_FALL          = 0x00010000;
        const TICK              = 0x00020000;
        const SEE_PLAYER        = 0x00040000;
        const ENEMY_NOT_VISIBLE = 0x00080000;
        const HEAR_NOISE        = 0x00100000;
        const UPDATE_EYE_HEIGHT = 0x00200000;
        const SEE_MONSTER       = 0x00400000;
        const SPECIAL_HANDLING  = 0x00800000;
        const BOT_DESIREABILITY = 0x01000000;
        const NOTIFY_BUMP       = 0x02000000;
        const NOTIFY_LANDED     = 0x04000000;
        const NOTIFY_HIT_WALL   = 0x08000000;
        const PRE_BEGIN_PLAY    = 0x10000000;
        const POST_BEGIN_PLAY   = 0x20000000;
        const DESTRUCT_CLEAN_UP = 0x40000000;
        const ALL               = 0x80000000;
    }
}

serializable_bitflags!(Events);

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct FunctionMapEntry {
    pub name: ArchivedName,
    pub function: OptionalPackageObjectIndex,
}
