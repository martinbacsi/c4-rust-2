use std::collections::HashMap;
use std::sync::Arc;

use crate::game::*;

struct Referee {
    game_manager: Arc<MultiplayerGameManager<Player>>,
    command_manager: Arc<CommandManager>,
    game: Arc<Game>,
}

impl Referee {
    fn init(&self) {
        match self.game_manager.get_league_level() {
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
        }

        self.game.init();
        self.send_global_info();

        self.game_manager.set_frame_duration(500);
        self.game_manager.set_max_turns(Game::MAX_TURNS);
        self.game_manager.set_turn_max_time(50);
        self.game_manager.set_first_turn_max_time(1000);
    }

    fn abort(&self) {
        self.game_manager.end_game();
    }

    fn send_global_info(&self) {
        // Give input to players
        for player in self.game_manager.get_active_players() {
            for line in Serializer::serialize_global_info_for(&player, &self.game) {
                player.send_input_line(line);
            }
        }
    }

    fn game_turn(&self, turn: i32) {
        self.game.reset_game_turn_data();

        // Give input to players
        for player in self.game_manager.get_active_players() {
            for line in Serializer::serialize_frame_info_for(&player, &self.game) {
                player.send_input_line(line);
            }
            player.execute();
        }

        // Get output from players
        self.handle_player_commands();

        self.game.perform_game_update(turn);

        if self.game_manager.get_active_players().len() < 2 {
            self.abort();
        }
    }

    fn handle_player_commands(&self) {
        for player in self.game_manager.get_active_players() {
            match command_manager.parse_commands(player, player.get_outputs()) {
                Ok(()) => {}
                Err(e) => {
                    player.deactivate(format!("Error: {}", e));
                    game_manager.add_to_game_summary(format!(
                        "{} has not provided {} lines in time",
                        player.get_nickname_token(),
                        player.get_expected_output_lines()
                    ));
                }
            }
        }
    }

    fn on_end(&self) {
        self.game.on_end();
    }
}