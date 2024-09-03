// Open TT main module file, defines base structs and enums


pub mod board_object;
pub mod board;
pub mod game;


use std::{cmp::{max, min}, collections::HashMap};
use board_object::{BoardObject, INPASSABLE};


const PLAYER_MOVE_DIST :u16= 1;
const PLAYER_SHOOT_DIST :u16= 3;


// Represents a single game of Tank Tactics
pub struct  Game {
    pub starting_board: Board,
    pub current_board: Board,
    pub moves: Vec<Action>,
    pub game_state: GameState
}

impl Default for Game {
    fn default() -> Self {
        Self { 
            starting_board: Board { size_x: 0, size_y: 0, players: HashMap::new(), objects: HashMap::new() }, 
            current_board: Board { size_x: 0, size_y: 0, players: HashMap::new(), objects: HashMap::new() }, 
            moves: Vec::new(), 
            game_state: GameState::InProgress }
    }
}

// Represents a game board, consisting of living players and board objects
#[derive(Debug, Clone)]
pub struct Board {
    pub size_x : u16,
    pub size_y : u16,
    pub players : HashMap<u8, PlayerTank>, // Players are referenced by their ID
    pub objects : HashMap<BoardPos, BoardObject> // Board objects are refenced by their position, since they are static
}


pub struct Map {
    pub items : Vec<MapItem>,
    pub size_x : u16,
    pub size_y : u16
}


pub enum MapItem {
    BoardObjectItem(u8, BoardPos)
}


pub enum AccessError {
    CouldNotFindPlayer,
    PlayerAPInsufficient
}


// Represents a position on the game board as a tuple of values x and y
#[derive(Debug, PartialEq, Eq, Hash, Clone)]
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
#[derive(Debug, Clone)]
pub struct PlayerTank {
    pub position : BoardPos,
    pub hitpoints : u8,
    pub action_points : u8,
}

impl Default for PlayerTank {
    fn default() -> Self {
        Self { position: BoardPos(0, 0), hitpoints: 3, action_points: 0}
    }
}


// A generic enum to represent things that can be in a board postion
pub enum BoardThing {
    PlayerThing(u8), // Represents a player in a position, the first data point refers to the id of the player
    ObjectThing // Represents a board object
}


#[derive(Debug, PartialEq, Eq)]
pub enum PlayerHitResult {
    PlayerKilled,
    PlayerAlive
}


#[derive(Debug, PartialEq, Eq)]
pub enum BoardObjectHitResult {
    Destroyed,
    NoEffect
}

pub enum ActionError {
    OutOfBounds,
    SpaceOccupied,
    NoTargetFound,
    InvalidPlayerID,
    NotEnoughAP,
    TargetTooFar
}

pub enum Action {
    TankMove(u8, BoardPos),
    TankShoot(u8, BoardPos),
    TankGiveAP(u8, BoardPos),
}


#[derive(Debug, PartialEq, Eq)]
pub enum GameState {
    Pregame,
    InProgress, 
    GameWon(u8) // State representing a won game, the parameter is the id of the winning player
}