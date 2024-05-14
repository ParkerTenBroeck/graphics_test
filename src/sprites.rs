
#[derive(Default, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
pub struct Sprite {
    pub x: u16,
    pub y: u16,

    pub tx: u8,
    pub ty: u8,

    pub layer: u8,
    pub attribute: SpriteAttributes,
}

mycelium_bitfield::bitfield! {
    #[derive(Default, PartialEq, Eq)]
    pub struct SpriteAttributes<u8> {
        pub const HORIZONTAL: bool;
        pub const VERTICAL: bool;
        pub const ROTATION = 2;
        pub const XSIZE = 2;
        pub const YSIZE = 2;
    }
}
