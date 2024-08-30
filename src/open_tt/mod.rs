// Open TT main module file, defines base structs and enums
pub mod board_object;


use std::{cmp::{min, max}, collections::HashMap};
use board_object::BoardObject;


// Represents a game board, consisting of living players and board objects
pub struct Board {
    pub size_x : u16,
    pub size_y : u16,
    pub players : HashMap<u8, PlayerTank>, // Players are referenced by their ID
    pub objects : HashMap<BoardPos, BoardObject> // Board objects are refenced by their position, since they are static
}


// Represents a position on the game board as a tuple of values x and y
pub struct BoardPos(pub u16, pub u16);

impl BoardPos {
    pub fn get_grid_dist(&self, other : &BoardPos) -> u16 {
        max(
            max(self.0, other.0) -  min(self.0, other.0),
            max(self.1, other.1) -  min(self.1, other.1)
        )
    }
}


// Represents a player controlled tank, mostly a data container for position and attributes
pub struct PlayerTank {
    pub position : BoardPos,
    pub hitpoints : u8,
    pub action_points : u8,
}




