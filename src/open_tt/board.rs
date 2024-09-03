// Defines a game board
use std::{cmp::{max, min}, collections::HashMap};
use super::*;




impl Board {
    // Internal function, check if position is in bounds
    fn pos_in_bounds(&self, pos: &BoardPos) -> bool {
        pos.0 < self.size_x && pos.1 < self.size_y
    }

    fn player_exists(&self, p_id: &u8) -> bool {
        self.players.contains_key(p_id)
    }

    // Get everything at the board position to check if it is something that would prevent traverse
    // If nothing in it prevents traverse, or the position is empty, return true
    fn is_pos_traversable(&self, pos : &BoardPos) -> bool {
        match self.get_things_at_pos(pos)
            .iter()
            .map(|thing| match *thing { // Convert things at pos to determine wether they would block traverse
                BoardThing::ObjectThing => self.objects.get(pos).unwrap().type_flags & INPASSABLE != 0,
                BoardThing::PlayerThing(_) => false
            })
            .reduce(|acc, e| acc && e) {
                Some(t) => t,
                None => true
            }
    }

    fn is_pos_in_bounds(&self, pos : &BoardPos) -> bool {
        pos.0 < self.size_x && pos.1 < self.size_y
    }

    fn player_has_ap(&self, p_id : &u8) -> bool {
        return match self.players.get(p_id) {
            Some(p) => p.action_points > 0,
            None => false
        }
    }

    // Internal function, get a list of all the board things at the given position 
    fn get_things_at_pos(&self, pos: &BoardPos) -> Vec<BoardThing> {
        let mut out: Vec<BoardThing> = Vec::new();
        
        // Get the static object at the position if one exists
        match self.objects.get(pos) {
            Some(object) => {out.push(
                BoardThing::ObjectThing
            );},
            None => {}
        }
        
        // Add any players in this position
        for (id, tank) in self.players.iter() {
            if tank.position == *pos {
                out.push(BoardThing::PlayerThing(*id));
            }
        }

        out
    }

    fn get_player_id_at_pos(&self, pos: &BoardPos) -> Option<u8> {
        let things = self.get_things_at_pos(pos);
        for thing in things {
            match thing {
                BoardThing::PlayerThing(id) => return Some(id),
                _ => {}
            };
        }
        return None;
    }

    // Take one action point from the player at the target position
    fn take_ap_from_player(&mut self, p_id: &u8) -> Result<(), AccessError> {
        let mut player = match self.players.get_mut(p_id) {
            Some(p) => p,
            None => {return Err(AccessError::CouldNotFindPlayer);}
        };

        if player.action_points == 0 {
            return Err(AccessError::PlayerAPInsufficient);
        }

        player.action_points -= 1;

        Ok(())
    }

    // Damage all things at the given position, returns a list of board things that have been destroyed
    fn damage_things_at_board_pos(&mut self, pos: &BoardPos) -> Vec<BoardThing> {
        let things = self.get_things_at_pos(pos);
        let mut out: Vec<BoardThing> = Vec::new();
        for board_thing in things {
            match board_thing {
                BoardThing::PlayerThing(p_id) => {
                    let kill = self.damage_and_kill_player(&p_id);
                    if kill == PlayerHitResult::PlayerKilled {
                        out.push(BoardThing::PlayerThing(p_id));
                    }
                }
                BoardThing::ObjectThing => {
                    let kill = self.damage_and_destroy_board_pos(pos);
                    if kill == BoardObjectHitResult::Destroyed {
                        out.push(BoardThing::ObjectThing);
                    }
                }
            }
        }
        return out;
    }

    // Damages the player and removes it from living player map if killed
    // Should only be called internally, assumes that the given player ID is valid
    fn damage_and_kill_player(&mut self, p_id: &u8) -> PlayerHitResult {
        let mut player = self.players.remove(p_id).unwrap();
        player.hitpoints -= 1;
        if player.hitpoints == 0 {
            drop(player);
            return PlayerHitResult::PlayerKilled;
        }
        self.players.insert(*p_id, player);

        PlayerHitResult::PlayerAlive
    }

    // Attempts to destroy the board thing at the given position
    fn damage_and_destroy_board_pos(&mut self, pos : &BoardPos) -> BoardObjectHitResult {
        // Check if anything even is at the position
        let pos_flags = match self.objects.get(pos) {
            Some(object) => object.type_flags,
            None => {return BoardObjectHitResult::NoEffect;}
        };

        // Check if the thing at the position is destructable
        if pos_flags & board_object::DESTRUCTABLE == 0 {
            return BoardObjectHitResult::NoEffect;
        }

        // Clear the board position
        let _ = self.objects.remove(pos);
        return BoardObjectHitResult::Destroyed;
    }

    // Tries to move a player to the target position
    fn apply_move_action(&mut self, p_id : &u8, t_pos : &BoardPos) -> Result<(), ActionError> {
        if !self.is_pos_traversable(t_pos) {
            return Err(ActionError::SpaceOccupied);
        }

        if !self.is_pos_in_bounds(t_pos) {
            return Err(ActionError::OutOfBounds);
        }

        let p_pos = match self.players.get(p_id) {
            None => {return Err(ActionError::InvalidPlayerID);},
            Some(p) => &p.position
        };

        if p_pos.get_grid_dist(t_pos) > PLAYER_MOVE_DIST {
            return Err(ActionError::TargetTooFar);
        }

        let take_result = self.take_ap_from_player(p_id);
        match take_result {
            Ok(_) => {}
            Err(e) => match e {
                AccessError::CouldNotFindPlayer => {return Err(ActionError::InvalidPlayerID);}
                AccessError::PlayerAPInsufficient => {return Err(ActionError::NotEnoughAP);}
            }
        }

        self.players.get_mut(p_id).unwrap().position = t_pos.clone();

        return Ok(());
    }

    fn apply_shoot_action(&mut self, p_id : &u8, t_pos : &BoardPos) -> Result<(), ActionError> {
        if !self.is_pos_in_bounds(t_pos) {
            return Err(ActionError::OutOfBounds);
        }

        let p_pos = match self.players.get(p_id) {
            None => {return Err(ActionError::InvalidPlayerID);},
            Some(p) => &p.position
        };

        if p_pos.get_grid_dist(t_pos) > PLAYER_SHOOT_DIST {
            return Err(ActionError::TargetTooFar);
        }

        let take_result = self.take_ap_from_player(p_id);
        match take_result {
            Ok(_) => {}
            Err(e) => match e {
                AccessError::CouldNotFindPlayer => {return Err(ActionError::InvalidPlayerID);}
                AccessError::PlayerAPInsufficient => {return Err(ActionError::NotEnoughAP);}
            }
        }

        let _ = self.damage_things_at_board_pos(t_pos);

        return Ok(());
    }

    fn apply_give_ap_action(&mut self, p_id : &u8, t_pos : &BoardPos) -> Result<(), ActionError> {
        if !self.is_pos_in_bounds(t_pos) {
            return Err(ActionError::OutOfBounds);
        }

        let p_pos = match self.players.get(p_id) {
            None => {return Err(ActionError::InvalidPlayerID);},
            Some(p) => &p.position
        };

        if p_pos.get_grid_dist(t_pos) > PLAYER_SHOOT_DIST {
            return Err(ActionError::TargetTooFar);
        }

        let target_player_id = match self.get_player_id_at_pos(t_pos) {
            Some(i) => i,
            None => {return Err(ActionError::NoTargetFound)}
        };

        let take_result = self.take_ap_from_player(p_id);
        match take_result {
            Ok(_) => {}
            Err(e) => match e {
                AccessError::CouldNotFindPlayer => {return Err(ActionError::InvalidPlayerID);}
                AccessError::PlayerAPInsufficient => {return Err(ActionError::NotEnoughAP);}
            }
        }

        self.players.get_mut(&target_player_id).unwrap().action_points += 1;

        return Ok(());
    }

    pub fn try_do_action(&mut self, action : &Action) -> Result<(), ActionError> {
        match action {
            Action::TankGiveAP(p_id, t_pos) => self.apply_give_ap_action(&p_id, &t_pos),
            Action::TankMove(p_id, t_pos) => self.apply_move_action(&p_id, &t_pos),
            Action::TankShoot(p_id, t_pos) => self.apply_shoot_action(&p_id, &t_pos)
        }
    }

    pub fn get_game_state(&self) -> GameState {
        if self.players.len() == 1 {
            // Hack to get the only value of a size one iterrator
            let _ = self.players.keys().map(|id| return GameState::GameWon(*id));
        }
        return GameState::InProgress;
    }
}