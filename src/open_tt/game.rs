use super::*;
use board;
use rand::{thread_rng, seq::SliceRandom};

// Implementation file for Game Struct
enum BoardReconstructionError {
    TurnOutOfBounds,
    MoveError(u16, ActionError)
}

enum MoveError {
    ActionError(ActionError),
    GameIsOver
}

impl Game {
    fn new(player_count : &u8, map : &Map) -> Game {
        let obstacles : HashMap<BoardPos, BoardObject> = HashMap::from_iter(
            map.items.iter()
            .map(|i| match i {
                MapItem::BoardObjectItem(t, p) => (p.clone(), BoardObject{type_flags : *t})
            })
        );

        let mut spawnpoints : Vec<BoardPos> = Vec::new();

        for x in (0..map.size_x) {
            for y in (0..map.size_y) {
                if match obstacles.get(&BoardPos(x, y)) {
                    None => true,
                    Some(o) => o.type_flags & INPASSABLE != 0
                } {
                    spawnpoints.push(BoardPos(x, y));
                }
            }
        }

        let mut players : HashMap<u8, PlayerTank> = HashMap::new();
        spawnpoints.shuffle(&mut thread_rng());

        for id in 0..*player_count {
            players.insert(id, PlayerTank{position: spawnpoints.pop().unwrap(), ..Default::default()});
        } 

        let board = Board {
            size_x: map.size_x,
            size_y: map.size_y,
            players: players,
            objects: obstacles
        };

        Self { starting_board: board.clone(), current_board: board, ..Default::default() }
    }

    // Reconstructs the board state after a given number of turns
    fn get_board_at_turn(&self, turn_num: u16) -> Result<Board, BoardReconstructionError> {
        if usize::from(turn_num) >= self.moves.len() {
            return Err(BoardReconstructionError::TurnOutOfBounds)
        }


        let mut new_board = self.starting_board.clone();

        for t_ind in 0..turn_num {
            let action = self.moves.get(usize::from(turn_num)).unwrap(); 
            let result = new_board.try_do_action(action);
            match result {
                Err(e) => {return Err(BoardReconstructionError::MoveError(t_ind, e));},
                _ => {}
            };
        }

        return Ok(new_board);
    }

    fn do_action(&mut self, action: Action) -> Result<(), MoveError>{
        if self.game_state != GameState::InProgress {
            return Err(MoveError::GameIsOver);
        }
        let result = self.current_board.try_do_action(&action);
        match result {
            Err(e) => {return Err(MoveError::ActionError(e));},
            Ok(()) => {
                self.moves.push(action);
            }
        };
        self.game_state = self.current_board.get_game_state();
        return Ok(());
    }
}