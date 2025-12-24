#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Quirks {
    /// If true, logic ops reset VF.
    pub vf_reset: bool,
    /// If true, FX55/FX65 increments I. If false, I stays same.
    pub memory_increment: bool,
    /// If true, clipping is handled differently (optional)
    pub clipping: bool,
    /// Drawing sprites to the display waits for the vertical blank interrupt,
    /// limiting their speed to max 60 sprites per second.
    pub display_wait: bool,
    /// If true, Vx = Vy >> 1. If false, Vx = Vx >> 1
    pub shift_vy: bool,
    /// (Bnnn) doesn't use v0, but vX instead where X is the highest nibble of nnn
    pub jumping: bool,
}

impl Quirks {
    pub const MODERN: Quirks = Quirks {
        vf_reset: true,
        memory_increment: true,
        clipping: true,
        display_wait: false,
        shift_vy: true,
        jumping: true,
    };
}
