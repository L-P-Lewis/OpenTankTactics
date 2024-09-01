use super::*;
use board;

// Implementation file for Game Struct
enum BoardReconstructionError {
    TurnOutOfBounds,
    MoveError(u16, ActionError)
}


impl Game {
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

    fn do_action(&mut self, action: Action) -> Result<(), ActionError>{
        let result = self.current_board.try_do_action(&action);
        match result {
            Err(e) => {return Err(e);},
            Ok(()) => {
                self.moves.push(action);
            }
        };
        return Ok(());
    }
}