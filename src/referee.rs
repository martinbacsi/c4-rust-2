use std::collections::HashMap;
use std::sync::Arc;

use crate::game::*;

struct Referee {
    game: Game,
}

impl Referee {
    fn new() -> Referee {
        /*match self.game_manager.get_league_level() {
            1 => {
                Game::ENABLE_UGLIES = false;
                Game::FISH_WILL_FLEE = false;
                Game::DRONES_PER_PLAYER = 1;
                Game::SIMPLE_SCANS = true;
                Game::FISH_WILL_MOVE = true;
            }
            2 => {
                Game::ENABLE_UGLIES = false;
                Game::FISH_WILL_FLEE = false;
                Game::DRONES_PER_PLAYER = 1;
            }
            3 => {
                Game::ENABLE_UGLIES = false;
            }
            _ => {}
        }*/

        let mut ret = Referee {
            game: Game::new()
        };
        ret
    }

    fn game_turn(&mut self, turn: i32) {
        self.game.reset_game_turn_data();

        // Give input to players
        for player in &self.game.players {
            //GET COMMAND
        }
    }
}