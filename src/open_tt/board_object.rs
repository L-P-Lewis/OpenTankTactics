use super::BoardPos;

// Flags for object properties
pub const INPASSABLE :u8= 0b00000001;
pub const BLOCK_SIGHT :u8= 0b00000010;
pub const DESTRUCTABLE :u8= 0b00000100;


// Some basic object types
pub const FOREST :u8= DESTRUCTABLE + BLOCK_SIGHT;
pub const WATER :u8= INPASSABLE;


// Represents a semi-static board object
#[derive(Debug, Clone)]
pub struct BoardObject {
    pub type_flags : u8
}