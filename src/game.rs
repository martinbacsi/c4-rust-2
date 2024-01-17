use std::collections::{HashMap, HashSet};
use rand::prelude::*;

use crate::{ugly::*, fish::*, player::*, scan::*, vector::*, collision::*, entity::* };

// Assuming you already have the necessary structs and enums from previous translations

#[derive(Debug)]
pub struct Game {
    random: dyn rand::Rng,
    players: Vec<Player>,
    fishes: Vec<Fish>,
    uglies: Vec<Ugly>,
    first_to_scan: HashMap<Scan, i32>,
    first_to_scan_temp: HashMap<Scan, i32>,
    first_to_scan_all_fish_of_color: HashMap<i32, i32>,
    first_to_scan_all_fish_of_color_temp: HashMap<i32, i32>,
    first_to_scan_all_fish_of_type: HashMap<FishType, i32>,
    first_to_scan_all_fish_of_type_temp: HashMap<FishType, i32>,
    game_turn: i32,
}

impl Game {
    pub const COLORS: [&'static str; 4] = ["pink", "yellow", "green", "blue"];
    pub const WIDTH: i32 = 10000;
    pub const HEIGHT: i32 = 10000;
    pub const DRONES_PER_PLAYER: i32 = 2;
    pub const UGLY_UPPER_Y_LIMIT: i32 = 2500;
    pub const DRONE_UPPER_Y_LIMIT: i32 = 0;
    pub const DRONE_START_Y: i32 = 500;
    pub const COLORS_PER_FISH: i32 = 4;
    pub const DRONE_MAX_BATTERY: i32 = 30;
    pub const LIGHT_BATTERY_COST: i32 = 5;
    pub const DRONE_BATTERY_REGEN: i32 = 1;
    pub const DRONE_MAX_SCANS: i32 = i32::MAX;
    pub const DARK_SCAN_RANGE: i32 = 800;
    pub const LIGHT_SCAN_RANGE: i32 = 2000;
    pub const UGLY_EAT_RANGE: i32 = 300;
    pub const DRONE_HIT_RANGE: i32 = 200;
    pub const FISH_HEARING_RANGE: i32 = (Game::DARK_SCAN_RANGE + Game::LIGHT_SCAN_RANGE) / 2;
    pub const DRONE_MOVE_SPEED: i32 = 600;
    pub const DRONE_SINK_SPEED: i32 = 300;
    pub const DRONE_EMERGENCY_SPEED: i32 = 300;
    pub const DRONE_MOVE_SPEED_LOSS_PER_SCAN: f64 = 0.0;
    pub const FISH_SWIM_SPEED: i32 = 200;
    pub const FISH_AVOID_RANGE: i32 = 600;
    pub const FISH_FLEE_SPEED: i32 = 400;
    pub const UGLY_ATTACK_SPEED: i32 = (Game::DRONE_MOVE_SPEED as f64 * 0.9) as i32;
    pub const UGLY_SEARCH_SPEED: i32 = (Game::UGLY_ATTACK_SPEED as f64 / 2.0) as i32;
    pub const FISH_X_SPAWN_LIMIT: i32 = 1000;
    pub const FISH_SPAWN_MIN_SEP: i32 = 1000;
    pub const ALLOW_EMOJI: bool = true;
    pub const CENTER: Vector = Vector {
        x: (Game::WIDTH - 1) as f64 / 2.0,
        y: (Game::HEIGHT - 1) as f64 / 2.0,
    };
    pub const MAX_TURNS: i32 = 201;
    pub const ENABLE_UGLIES: bool = true;
    pub const FISH_WILL_FLEE: bool = true;
    pub const FISH_WILL_MOVE: bool = true;
    pub const SIMPLE_SCANS: bool = false;
    pub const chased_fish_count: [i32; 4] = [0; 4];
    pub const times_aggroed: [i32; 4] = [0; 4];
    pub const max_turns_spent_with_scan: [i32; 4] = [0; 4];
    pub const max_y: [i32; 4] = [0; 4];
    pub const turn_saved_fish: [[i32; 4]; 4] = [[0; 4]; 4];
    pub const drones_eaten: i32 = 0;
    pub const fish_scanned: i32 = 0;
    pub const ENTITY_COUNT: i32 = 0;

    pub fn init(&mut self) {
        Game::ENTITY_COUNT = 0;
        self.players = self.game_manager.get_players();
        self.random =rand::thread_rng();
        self.viewer_events = Vec::new();
        self.game_turn = 1;
        self.init_players();
        self.init_fish();
        self.init_uglies();

        Game::chased_fish_count = [0; 2];
        Game::max_turns_spent_with_scan = [0; 2];
        Game::max_y = [0; 2];
        Game::turn_saved_fish = [[-1; 12], [-1; 12]];
        Game::times_aggroed = [0; 2];
        Game::drones_eaten = 0;
        Game::fish_scanned = 0;

        for player in &self.players {
            if Game::SIMPLE_SCANS {
                player.visible_fishes.extend_from_slice(&self.fishes);
            }
        }
    }

    fn all_entities(players: &Vec<Player>, fishes: &Vec<Fish>, uglies: &Vec<Ugly>) -> Box<dyn Iterator<Item = dyn Entity>> {
        Box::new(
            players.iter().flat_map(|s| s.drones.iter())
                .chain(fishes.iter())
                .chain(uglies.iter())
        )
    }

    fn init_uglies(random: &mut dyn Rng, entity_count: &mut i32) -> Vec<Ugly> {
        let mut uglies = Vec::new();

        let ugly_count = if Game::ENABLE_UGLIES { 1 + random.gen::<i32>() % 3 } else { 0 };

        for _ in 0..ugly_count {
            let x = random.gen::<i32>() % (Game::WIDTH / 2);

            let y = Game::HEIGHT / 2 + random.gen::<i32>() % (Game::HEIGHT / 2);
            for k in 0..2 {
                let mut ugly = Ugly::new(x, y, *entity_count);
                if k == 1 {
                    ugly.pos = ugly.pos.hsymmetric(Game::CENTER.x);
                }

                uglies.push(ugly);
                *entity_count += 1;
            }
        }

        uglies
    }

    fn init_fish(random: &mut dyn Rng, entity_count: &mut i32) -> Vec<Fish> {
        let mut fishes = Vec::new();

        for col in (0..Game::COLORS_PER_FISH).step_by(2) {
            for type_idx in 0..FishType::variants().len() {
                let mut position_found = false;
                let mut iterations = 0;
                let mut x = 0;
                let mut y = 0;

                let mut low_y = Game::HEIGHT / 4;
                let mut high_y = Game::HEIGHT;

                while !position_found {
                    x = random.gen::<i32>() % (Game::WIDTH - Game::FISH_X_SPAWN_LIMIT * 2) + Game::FISH_X_SPAWN_LIMIT;
                    if type_idx == 0 {
                        y = 1 * Game::HEIGHT / 4 + Game::FISH_SPAWN_MIN_SEP;
                        low_y = 1 * Game::HEIGHT / 4;
                        high_y = 2 * Game::HEIGHT / 4;
                    } else if type_idx == 1 {
                        y = 2 * Game::HEIGHT / 4 + Game::FISH_SPAWN_MIN_SEP;
                        low_y = 2 * Game::HEIGHT / 4;
                        high_y = 3 * Game::HEIGHT / 4;
                    } else {
                        y = 3 * Game::HEIGHT / 4 + Game::FISH_SPAWN_MIN_SEP;
                        low_y = 3 * Game::HEIGHT / 4;
                        high_y = 4 * Game::HEIGHT / 4;
                    }
                    y += random.gen::<i32>() % (Game::HEIGHT / 4 - Game::FISH_SPAWN_MIN_SEP * 2);

                    let final_x = x;
                    let final_y = y;
                    let too_close = fishes.iter().any(|other| other.get_pos().in_range(&Vector::new(final_x, final_y), Game::FISH_SPAWN_MIN_SEP));
                    let too_close_to_center = (Game::CENTER.x - x).abs() <= Game::FISH_SPAWN_MIN_SEP;
                    if !too_close && !too_close_to_center || iterations > 100 {
                        position_found = true;
                    }
                    iterations += 1;
                }
                let mut f = Fish::new(x, y, FishType::variants()[type_idx], col, *entity_count, low_y, high_y);

                let snapped = (random.gen::<i32>() % 7) as f64 * std::f64::consts::FRAC_PI_4;
                let direction = Vector::new(snapped.cos(), snapped.sin());

                if Game::FISH_WILL_MOVE {
                    f.speed = direction.mult(Game::FISH_SWIM_SPEED).round();
                }

                fishes.push(f);
                *entity_count += 1;

                let other_pos = f.pos.hsymmetric(Game::CENTER.x);
                let mut o = Fish::new(other_pos.x, other_pos.y, FishType::variants()[type_idx], col + 1, *entity_count, f.low_y, f.high_y);
                o.speed = f.speed.hsymmetric();
                fishes.push(o);
                *entity_count += 1;
            }
        }

        fishes
    }
        
    fn init_players(players: &mut Vec<Player>, game_manager: &MultiplayerGameManager<Player>, entity_count: &mut i32) {
        let idxs = [0, 2, 1, 3];
        let mut idx_idx = 0;
        for _ in 0..Game::DRONES_PER_PLAYER {
            let x = Game::WIDTH / (Game::DRONES_PER_PLAYER * 2 + 1) * (idxs[idx_idx] + 1);
            idx_idx += 1;
            for player in game_manager.get_active_players() {
                let mut drone = Drone::new(x as f64, Game::DRONE_START_Y as f64, *entity_count, player);

                if player.get_index() == 1 {
                    drone.pos = drone.pos.hsymmetric(Game::CENTER.x);
                }

                player.drones.push(drone);
                *entity_count += 1;
            }
        }
    }

    fn reset_game_turn_data(players: &mut Vec<Player>, viewer_events: &mut Vec<EventData>, animation: &mut Animation) {
        viewer_events.clear();
        animation.reset();
        for player in players.iter_mut() {
            player.reset();
        }
    }

    fn update_ugly_target(ugly: &mut Ugly, players: &Vec<Player>, times_aggroed: &mut Vec<i32>) -> bool {
        let targetable_drones: Vec<&Drone> = players.iter()
            .flat_map(|p| p.drones.iter())
            .filter(|drone| {
                let scan_range = if drone.is_light_on() {
                    Game::LIGHT_SCAN_RANGE
                } else {
                    Game::DARK_SCAN_RANGE
                };
                drone.pos.in_range(&ugly.pos, scan_range) && !drone.is_dead_or_dying()
            })
            .collect();

        if !targetable_drones.is_empty() {
            let closest_targets = get_closest_to(&ugly.pos, targetable_drones.iter().cloned()); // Implement get_closest_to function
            ugly.target = Some(closest_targets.get_mean_pos().clone());
            for d in closest_targets.list.iter() {
                times_aggroed[d.owner.get_index()] += 1;
            }
            true
        } else {
            ugly.target = None;
            false
        }
    }

    fn move_entities(
        players: &mut Vec<Player>,
        uglies: &mut Vec<Ugly>,
        fishes: &mut Vec<Fish>,
        drones_eaten: &mut i32,
        chased_fish_count: &mut [i32; 2],
        game_summary_manager: &mut GameSummaryManager,
    ) {
        // Move drones and handle collisions with uglies
        for player in players.iter_mut() {
            for drone in player.drones.iter_mut() {
                if drone.dead {
                    continue;
                }
            
                // NOTE: the collision code does not take into account the snap to map borders
                for ugly in uglies.iter() {
                    let col = get_collision(drone, ugly);
                    if col.happened() {
                        drone.dying = true;
                        drone.scans.clear();
                        drone.die_at = col.t;

                        game_summary_manager.add_player_summary(
                            player.get_nickname_token(),
                            format!(
                                "{}'s drone {} is hit by monster {}!",
                                player.get_nickname_token(),
                                drone.id,
                                ugly.id
                            ),
                        );

                        *drones_eaten += 1;
                        // If two uglies hit the drone, let's just keep the first collision, it matters not.
                        break;
                    }
                }
            }
        }

        // Move drones and snap to drone zone
        for player in players.iter_mut() {
            for drone in player.drones.iter_mut() {
                let speed = drone.get_speed();
                drone.pos = drone.pos.add(speed);
                snap_to_drone_zone(drone);
            }
        }

        // Move fishes and snap to fish zone
        for fish in fishes.iter_mut() {
            fish.pos = fish.pos.add(fish.get_speed());
            snap_to_fish_zone(fish);
        }

        // Remove fishes that went out of bounds
        let fish_to_remove: Vec<Fish> = fishes
            .iter()
            .filter(|fish| fish.get_pos().x > Game::WIDTH - 1.0 || fish.get_pos().x < 0.0)
            .cloned()
            .collect();

        for fish in &fish_to_remove {
            if let Some(fleeing_player) = fish.fleeing_from_player {
                chased_fish_count[fleeing_player] += 1;
            }
        }

        fishes.retain(|fish| !fish_to_remove.contains(fish));

        // Reset fleeing information for remaining fishes
        for fish in fishes.iter_mut() {
            fish.fleeing_from_player = None;
        }

        // Move uglies and snap to ugly zone
        for ugly in uglies.iter_mut() {
            ugly.pos = ugly.pos.add(ugly.speed);
            snap_to_ugly_zone(ugly);
        }
    }

    fn snap_to_ugly_zone(ugly: &mut Ugly) {
        if ugly.pos.y > Game::HEIGHT - 1.0 {
            ugly.pos = Vector::new(ugly.pos.x, Game::HEIGHT - 1.0);
        } else if ugly.pos.y < Game::UGLY_UPPER_Y_LIMIT as f64 {
            ugly.pos = Vector::new(ugly.pos.x, Game::UGLY_UPPER_Y_LIMIT as f64);
        }
    }

    fn update_ugly_speeds(uglies: &mut Vec<Ugly>, fishes: &Vec<Fish>) {
        for ugly in uglies.iter_mut() {
            if let Some(target) = ugly.target {
                let attack_vec = Vector::new(ugly.pos, target);
                if attack_vec.length() > Game::UGLY_ATTACK_SPEED as f64 {
                    ugly.speed = attack_vec.normalize().mult(Game::UGLY_ATTACK_SPEED as f64).round();
                }
            } else {
                if ugly.speed.length() > Game::UGLY_SEARCH_SPEED as f64 {
                    ugly.speed = ugly.speed.normalize().mult(Game::UGLY_SEARCH_SPEED as f64).round();
                }

                if !ugly.speed.is_zero() {
                    let closest_uglies = get_closest_to(&ugly.pos, uglies.iter().filter(|u| **u != *ugly));
                    if !closest_uglies.list.is_empty() && closest_uglies.distance <= Game::FISH_AVOID_RANGE as f64 {
                        let avoid = closest_uglies.get_mean_pos();
                        let avoid_dir = Vector::new(avoid, ugly.pos).normalize();
                        if !avoid_dir.is_zero() {
                            ugly.speed = avoid_dir.mult(Game::FISH_SWIM_SPEED as f64).round();
                        }
                    }
                }

                let next_pos = ugly.pos.add(ugly.speed);

                if (next_pos.x < 0.0 && next_pos.x < ugly.pos.x) || (next_pos.x > Game::WIDTH - 1.0 && next_pos.x > ugly.pos.x) {
                    ugly.speed = ugly.speed.hsymmetric();
                }

                if (next_pos.y < Game::UGLY_UPPER_Y_LIMIT as f64 && next_pos.y < ugly.pos.y)
                    || (next_pos.y > Game::HEIGHT - 1.0 && next_pos.y > ugly.pos.y)
                {
                    ugly.speed = ugly.speed.vsymmetric();
                }
            }
        }
    }

    fn update_ugly_targets(uglies: &mut Vec<Ugly>) {
        for ugly in uglies.iter_mut() {
            let found_target = update_ugly_target(ugly);
            ugly.found_target = found_target;
        }
    }

    fn update_fish(fishes: &mut Vec<Fish>, players: &Vec<Player>) {
        for fish in fishes.iter_mut() {
            fish.is_fleeing = false;

            let mut flee_from: Option<Vector> = None;
            if Game::FISH_WILL_FLEE {
                let closest_drones = get_closest_to(
                    &fish.pos,
                    players.iter().flat_map(|p| p.drones.iter().filter(|d| d.is_engine_on() && !d.dead)),
                );

                if !closest_drones.list.is_empty() && closest_drones.distance <= Game::FISH_HEARING_RANGE as f64 {
                    flee_from = Some(closest_drones.get_mean_pos().clone());
                    let mut fleeing_from_player: Option<usize> = None;
                    for d in closest_drones.list.iter() {
                        let idx = d.owner.get_index();
                        if fleeing_from_player.is_none() || fleeing_from_player == Some(idx) {
                            fleeing_from_player = Some(idx);
                        } else {
                            fleeing_from_player = Some(usize::MAX);
                        }
                    }
                    fish.fleeing_from_player = fleeing_from_player;
                }
            }

            if let Some(flee_from_vec) = flee_from {
                let flee_dir = Vector::new(flee_from_vec, fish.pos).normalize();
                let flee_vec = flee_dir.mult(Game::FISH_FLEE_SPEED as f64);
                fish.speed = flee_vec.round();
                fish.is_fleeing = true;
            } else {
                let swim_vec = fish.speed.normalize().mult(Game::FISH_SWIM_SPEED as f64);
                let closest_fishes = get_closest_to(&fish.pos, fishes.iter().filter(|f| **f != *fish));

                if !closest_fishes.list.is_empty() && closest_fishes.distance <= Game::FISH_AVOID_RANGE as f64 {
                    let avoid = closest_fishes.get_mean_pos();
                    let avoid_dir = Vector::new(avoid, fish.pos).normalize();
                    fish.speed = avoid_dir.mult(Game::FISH_SWIM_SPEED as f64);
                }

                let next_pos = fish.pos.add(&fish.speed);

                if (next_pos.x < 0.0 && next_pos.x < fish.pos.x)
                    || (next_pos.x > Game::WIDTH - 1.0 && next_pos.x > fish.pos.x)
                {
                    fish.speed = fish.speed.hsymmetric();
                }

                let y_highest = f64::min(Game::HEIGHT as f64 - 1.0, fish.high_y as f64);

                if (next_pos.y < fish.low_y as f64 && next_pos.y < fish.pos.y)
                    || (next_pos.y > y_highest && next_pos.y > fish.pos.y)
                {
                    fish.speed = fish.speed.vsymmetric();
                }
                fish.speed = fish.speed.epsilon_round().round();
            }
        }
    }

    fn snap_to_fish_zone(fish: &mut Fish) {
        if fish.pos.y > (Game::HEIGHT - 1) as f64 {
            fish.pos = Vector::new(fish.pos.x, (Game::HEIGHT - 1) as f64);
        } else if fish.pos.y > fish.high_y as f64 {
            fish.pos = Vector::new(fish.pos.x, fish.high_y as f64);
        } else if fish.pos.y < fish.low_y as f64 {
            fish.pos = Vector::new(fish.pos.x, fish.low_y as f64);
        }
    }

    fn update_drones(players: &mut Vec<Player>, height: f64, width: f64) {
        for player in players.iter_mut() {
            for drone in player.drones.iter_mut() {
                let move_speed =
                    (Game::DRONE_MOVE_SPEED - Game::DRONE_MOVE_SPEED * Game::DRONE_MOVE_SPEED_LOSS_PER_SCAN * drone.scans.len() as f64) as f64;

                if drone.dead {
                    let float_vec = Vector::new(0.0, -1.0).mult(Game::DRONE_EMERGENCY_SPEED as f64);
                    drone.speed = float_vec;
                } else if let Some(move_pos) = &drone.move {
                    let move_vec = Vector::new(drone.pos, *move_pos);

                    if move_vec.length() > move_speed {
                        drone.speed = move_vec.normalize().mult(move_speed).round();
                    } else {
                        drone.speed = move_vec.round();
                    }
                } else if drone.pos.y < height - 1.0 {
                    let sink_vec = Vector::new(0.0, 1.0).mult(Game::DRONE_SINK_SPEED as f64);
                    drone.speed = sink_vec;
                }
            }
        }
    }

    fn snap_to_drone_zone(drone: &mut Drone, height: f64, width: f64) {
        if drone.pos.y > height - 1.0 {
            drone.pos = Vector::new(drone.pos.x, height - 1.0);
        } else if drone.pos.y < Game::DRONE_UPPER_Y_LIMIT as f64 {
            drone.pos = Vector::new(drone.pos.x, Game::DRONE_UPPER_Y_LIMIT as f64);
        }

        if drone.pos.x < 0.0 {
            drone.pos = Vector::new(0.0, drone.pos.y);
        } else if drone.pos.x >= width {
            drone.pos = Vector::new(width - 1.0, drone.pos.y);
        }
    }

    fn get_closest_to<T: Entity>(from: &Vector, target_stream: Vec<T>) -> Closest<T> {
        let mut targets = target_stream;
        let mut closests = Vec::new();
        let mut min_dist = 0.0;

        for t in targets.iter() {
            let dist = t.get_pos().sqr_euclidean_to(from);

            if closests.is_empty() || dist < min_dist {
                closests.clear();
                closests.push(t.clone());
                min_dist = dist;
            } else if dist == min_dist {
                closests.push(t.clone());
            }
        }

        Closest {
            list: closests,
            distance: min_dist.sqrt(),
        }
    }

    fn perform_game_update(&mut self, frame_idx: i32) {
        self.clear_player_info();
        self.do_batteries();

        // Update speeds
        self.update_drones();

        // Move
        self.move_entities();

        // Target
        self.update_ugly_targets();

        // Scans
        self.do_scan();
        self.do_report();

        // Upkeep
        self.upkeep_drones();

        // Update speeds
        self.update_fish();
        self.update_ugly_speeds();

        if self.is_game_over() {
            self.compute_score_on_end();
            self.game_manager.end_game();
        }

        self.game_turn += 1;

        self.game_manager.add_to_game_summary(&self.game_summary_manager.to_string());
        self.game_summary_manager.clear();

        let frame_time = self.animation.compute_events();
        self.game_manager.set_frame_duration(frame_time);
    }

    fn clear_player_info(&mut self) {
        for player in self.players.iter_mut() {
            player.visible_fishes.clear();
        }
    }

    fn upkeep_drones(&mut self) {
        for player in self.players.iter_mut() {
            for drone in player.drones.iter_mut() {
                if drone.dying {
                    drone.dead = true;
                    drone.dying = false;
                } else if drone.dead && drone.get_y() == Game::DRONE_UPPER_Y_LIMIT as f64 {
                    drone.dead = false;
                }

                // Stats
                if drone.scans.is_empty() {
                    drone.turns_spent_with_scan = 0;
                } else {
                    drone.turns_spent_with_scan += 1;
                    if drone.turns_spent_with_scan > drone.max_turns_spent_with_scan {
                        drone.max_turns_spent_with_scan = drone.turns_spent_with_scan;
                    }
                }
                if drone.pos.y > drone.max_y as f64 {
                    drone.max_y = drone.pos.y as i32;
                }
            }
        }
    }

    fn do_batteries(&mut self) {
        for player in self.players.iter_mut() {
            for drone in player.drones.iter_mut() {
                if drone.dead {
                    drone.light_on = false;
                    continue;
                }

                if drone.light_switch && drone.battery >= Game::LIGHT_BATTERY_COST && !drone.dead {
                    drone.light_on = true;
                } else {
                    if drone.light_switch && !drone.dead {
                        self.game_summary_manager.add_player_summary(
                            &player.get_nickname_token(),
                            &format!("{}'s drone {} does not have enough battery to activate light", player.get_nickname_token(), drone.id),
                        );
                    }
                    drone.light_on = false;
                }

                if drone.is_light_on() {
                    drone.drain_battery();
                } else {
                    drone.recharge_battery();
                }
            }
        }
    }

    fn do_scan(&mut self) {
        for player in self.players.iter_mut() {
            for drone in player.drones.iter_mut() {
                if drone.is_dead_or_dying() {
                    continue;
                }

                let scannable_fish: Vec<&Fish> = self.fishes.iter()
                    .filter(|fish| fish.pos.in_range(&drone.pos, if drone.is_light_on() { Game::LIGHT_SCAN_RANGE } else { Game::DARK_SCAN_RANGE }))
                    .collect();

                for fish in scannable_fish.iter() {
                    player.visible_fishes.insert(**fish);

                    if drone.scans.len() < Game::DRONE_MAX_SCANS {
                        let scan = Scan::new(fish.clone());
                        if !player.scans.contains(&scan) {
                            if !drone.scans.contains(&scan) {
                                drone.scans.insert(scan.clone());
                                drone.fishes_scanned_this_turn.push(fish.id);
                            }
                        }
                    }
                }

                if Game::SIMPLE_SCANS {
                    player.visible_fishes.extend(self.fishes.iter());
                }

                if !drone.fishes_scanned_this_turn.is_empty() {
                    let summary_scan = drone.fishes_scanned_this_turn.iter().map(|&fish_scan| fish_scan.to_string()).collect::<Vec<String>>().join(",");
                    if drone.fishes_scanned_this_turn.len() == 1 {
                        self.game_summary_manager.add_player_summary(
                            &player.get_nickname_token(),
                            &format!(
                                "{}'s drone {} scans fish {}", player.get_nickname_token(), drone.id, drone.fishes_scanned_this_turn[0]
                            ),
                        );
                    } else {
                        self.game_summary_manager.add_player_summary(
                            &player.get_nickname_token(),
                            &format!(
                                "{}'s drone {} scans {} fish: {}", player.get_nickname_token(), drone.id, drone.fishes_scanned_this_turn.len(),
                                summary_scan
                            ),
                        );
                    }
                }
            }
        }
    }

    fn apply_scans_for_report(&mut self, player: &mut Player, drone: &Drone) -> bool {
        let mut points_scored = false;
        for scan in drone.scans.iter() {
            if !self.first_to_scan.contains_key(scan) {
                if self.first_to_scan_temp.contains_key(scan) {
                    // Opponent also completed this report this turn, nobody gets the bonus
                    self.first_to_scan_temp.remove(scan);
                } else {
                    self.first_to_scan_temp.insert(scan.clone(), player.get_index());
                }
            }
            let fish_index = scan.color * 3 + scan.fish_type as usize;
            self.turn_saved_fish[player.get_index() as usize][fish_index] = self.game_turn;
            self.fish_scanned += 1;
        }
        if !drone.scans.is_empty() {
            player.count_fish_saved.push(drone.scans.len() as i32);
        }
        points_scored |= player.scans.extend(drone.scans.drain());
        for other in player.drones.iter_mut() {
            if drone.id != other.id {
                other.scans.retain(|s| !drone.scans.contains(s));
            }
        }
        true
    }

    fn detect_first_to_combo_bonuses(&mut self, player: &mut Player) {
        for &fish_type in FishType::iter() {
            if !self.first_to_scan_all_fish_of_type.contains_key(&fish_type) {
                if self.player_scanned_all_fish_of_type(player, fish_type) {
                    if self.first_to_scan_all_fish_of_type_temp.contains_key(&fish_type) {
                        // Opponent also completed this report this turn, nobody gets the bonus
                        self.first_to_scan_all_fish_of_type_temp.remove(&fish_type);
                    } else {
                        self.first_to_scan_all_fish_of_type_temp.insert(fish_type, player.get_index());
                    }
                }
            }
        }

        for color in 0..Game::COLORS_PER_FISH {
            if !self.first_to_scan_all_fish_of_color.contains_key(&color) {
                if self.player_scanned_all_fish_of_color(player, color) {
                    if self.first_to_scan_all_fish_of_color_temp.contains_key(&color) {
                        // Opponent also completed this report this turn, nobody gets the bonus
                        self.first_to_scan_all_fish_of_color_temp.remove(&color);
                    } else {
                        self.first_to_scan_all_fish_of_color_temp.insert(color, player.get_index());
                    }
                }
            }
        }
    }

    fn do_report(&mut self) {
        for player in self.players.iter_mut() {
            let mut points_scored = false;
            for drone in player.drones.iter_mut() {
                if drone.is_dead_or_dying() {
                    continue;
                }
                if Game::SIMPLE_SCANS || (!drone.scans.is_empty() && drone.pos.y <= Game::DRONE_START_Y as f64) {
                    let drone_scored = self.apply_scans_for_report(player, drone);
                    points_scored |= drone_scored;
                    if drone_scored {
                        drone.did_report = true;
                    }
                }
            }
            if points_scored {
                self.game_manager.add_tooltip(player, &format!("{} reports their findings.", player.get_nickname_token()));
            }

            self.detect_first_to_combo_bonuses(player);
        }

        self.persist_first_to_scan_bonuses();
        for player in self.players.iter_mut() {
            player.points = self.compute_player_score(player);
        }
    }

    fn persist_first_to_scan_bonuses(&mut self) {
        let mut player_scans_map: HashMap<String, Vec<Scan>> = HashMap::new();

        for (scan, &player_index) in &self.first_to_scan_temp {
            self.first_to_scan.entry(scan.clone()).or_insert(player_index);

            let player_name = self.players[player_index as usize].get_nickname_token();
            player_scans_map.entry(player_name)
                .or_insert_with(|| Vec::new())
                .push(scan.clone());
        }

        for (player_name, player_scans) in player_scans_map {
            let summary_string = player_scans.iter()
                .map(|scan| scan.fish_id.to_string())
                .collect::<Vec<String>>()
                .join(", ");
        
            if player_scans.len() == 1 {
                self.game_summary_manager.add_player_summary(
                    player_name,
                    format!("{} was the first to save the scan of creature {}", player_name, summary_string)
                );
            } else {
                self.game_summary_manager.add_player_summary(
                    player_name,
                    format!(
                        "{} was the first to save the scans of {} creatures: {}", 
                        player_name, 
                        player_scans.len(), 
                        summary_string
                    )
                );
            }
        }

        for (fish_type, &player_index) in &self.first_to_scan_all_fish_of_type_temp {
            self.first_to_scan_all_fish_of_type.entry(*fish_type).or_insert(player_index);

            let player_name = self.players[player_index as usize].get_nickname_token();
            let fish_species = format!("{} ({})", fish_type as usize, fish_type.to_string().to_lowercase());
            self.game_summary_manager.add_player_summary(
                player_name,
                format!("{} saved the scans of every color of {} first", player_name, fish_species)
            );
        }

        for (color, &player_index) in &self.first_to_scan_all_fish_of_color_temp {
            self.first_to_scan_all_fish_of_color.entry(*color).or_insert(player_index);

            let player_name = self.players[player_index as usize].get_nickname_token();
            self.game_summary_manager.add_player_summary(
                player_name,
                format!(
                    "{} has saved the scans of every {} colored ({}) creature first", 
                    player_name, 
                    *color, 
                    COLORS[*color]
                )
            );
        }

        self.first_to_scan_temp.clear();
        self.first_to_scan_all_fish_of_color_temp.clear();
        self.first_to_scan_all_fish_of_type_temp.clear();
    }

    fn player_scanned(&self, player: &Player, fish: &Fish) -> bool {
        self.player_scanned_scan(player, &Scan::from(fish))
    }

    fn player_scanned_scan(&self, player: &Player, scan: &Scan) -> bool {
        player.scans.contains(scan)
    }

    fn has_scanned_all_remaining_fish(&self, player: &Player) -> bool {
        self.fishes.iter().all(|fish| self.player_scanned(player, fish))
    }

    fn has_fish_escaped(&self, scan: &Scan) -> bool {
        !self.fishes.iter().any(|fish| fish.color == scan.color && fish.fish_type == scan.fish_type)
    }

    fn is_fish_scanned_by_player_drone(&self, scan: &Scan, player: &Player) -> bool {
        player.drones.iter().any(|drone| drone.scans.contains(scan))
    }

    fn is_type_combo_still_possible(&self, p: &Player, fish_type: &FishType) -> bool {
        if self.player_scanned_all_fish_of_type(p, fish_type) {
            return false;
        }

        for color in 0..COLORS_PER_FISH {
            let scan = Scan::new(*fish_type, color);
            if self.has_fish_escaped(&scan) && !self.is_fish_scanned_by_player_drone(&scan, p) && !self.player_scanned_scan(p, &scan) {
                return false;
            }
        }
        true
    }

    fn is_color_combo_still_possible(&self, p: &Player, color: usize) -> bool {
        if self.player_scanned_all_fish_of_color(p, color) {
            return false;
        }

        for fish_type in FishType::values() {
            let scan = Scan::new(*fish_type, color);
            if self.has_fish_escaped(&scan) && !self.is_fish_scanned_by_player_drone(&scan, p) && !self.player_scanned_scan(p, &scan) {
                return false;
            }
        }
        true
    }

    fn compute_max_player_score(&self, p: &Player) -> i32 {
        let mut total = self.compute_player_score(p);
        let p2 = &self.players[1 - p.get_index()];

        for color in 0..COLORS_PER_FISH {
            for fish_type in FishType::values() {
                let scan = Scan::new(fish_type, color);
                if !self.player_scanned_scan(p, &scan) {
                    if self.is_fish_scanned_by_player_drone(&scan, p) || !self.has_fish_escaped(&scan) {
                        total += fish_type as i32 + 1;
                        if self.first_to_scan.get(&scan).map_or(true, |&val| val == -1) {
                            total += fish_type as i32 + 1;
                        }
                    }
                }
            }
        }

        for fish_type in FishType::values() {
            if self.is_type_combo_still_possible(p, &fish_type) {
                total += COLORS_PER_FISH as i32;
                if self.first_to_scan_all_fish_of_type.get(&fish_type).map_or(true, |&val| val != p2.get_index()) {
                    total += COLORS_PER_FISH as i32;
                }
            }
        }

        for color in 0..COLORS_PER_FISH {
            if self.is_color_combo_still_possible(p, color) {
                total += FishType::values().len() as i32;
                if self.first_to_scan_all_fish_of_color.get(&color).map_or(true, |&val| val != p2.get_index()) {
                    total += FishType::values().len() as i32;
                }
            }
        }

        total
    }

    fn is_game_over(&self) -> bool {
        if self.both_players_have_scanned_all_remaining_fish() {
            true
        } else {
            self.game_turn >= 200
                || self.compute_max_player_score(&self.players[0]) < self.players[1].points
                || self.compute_max_player_score(&self.players[1]) < self.players[0].points
        }
    }

    fn both_players_have_scanned_all_remaining_fish(&self) -> bool {
        self.players.iter().all(|player| self.has_scanned_all_remaining_fish(player))
    }

    fn player_scanned_all_fish_of_color(&self, player: &Player, color: i32) -> bool {
        FishType::values().iter().all(|&fish_type| self.player_scanned(player, &Scan::new(fish_type, color)))
    }

    fn compute_player_score(&self, player: &Player) -> i32 {
        let mut total = 0;
        for scan in &player.scans {
            total += scan.typ as i32 + 1;
            if self.first_to_scan.get(scan).map_or(false, |&val| val == player.get_index()) {
                total += scan.typ as i32 + 1;
            }
        }

        for fish_type in FishType::values() {
            if self.player_scanned_all_fish_of_type(player, fish_type) {
                total += COLORS_PER_FISH as i32;
            }
            if self.first_to_scan_all_fish_of_type.get(fish_type).map_or(false, |&val| val == player.get_index()) {
                total += COLORS_PER_FISH as i32;
            }
        }

        for color in 0..COLORS_PER_FISH {
            if self.player_scanned_all_fish_of_color(player, color) {
                total += FishType::values().len() as i32;
            }
            if self.first_to_scan_all_fish_of_color.get(&color).map_or(false, |&val| val == player.get_index()) {
                total += FishType::values().len() as i32;
            }
        }

        total
    }

    fn player_scanned_all_fish_of_type(&self, player: &Player, fish_type: FishType) -> bool {
        (0..COLORS_PER_FISH).all(|color| self.player_scanned(player, &Scan::new(fish_type, color)))
    }

    fn compute_score_on_end(&mut self) {
        for player in &mut self.players {
            for drone in &player.drones {
                self.apply_scans_for_report(player, &mut drone);
            }
            self.detect_first_to_combo_bonuses(player);
        }

        self.persist_first_to_scan_bonuses();

        for player in &mut self.players {
            if player.is_active() {
                let score = self.compute_player_score(player);
                player.set_score(score);
                player.points = score;
            } else {
                player.set_score(-1);
            }
        }
    }

    fn on_end(&mut self) {
        for player in &mut self.players {
            if player.is_active() {
                let score = self.compute_player_score(player);
                player.set_score(score);
                player.points = score;
            } else {
                player.set_score(-1);
            }
        }
    }

    fn get_collision(drone: &Drone, ugly: &Ugly) -> Collision {
        // Check instant collision
        if ugly.get_pos().in_range(&drone.get_pos(), DRONE_HIT_RANGE + UGLY_EAT_RANGE) {
            return Collision::new(0.0, ugly.clone(), drone.clone());
        }

        // Both units are motionless
        if drone.get_speed().is_zero() && ugly.get_speed().is_zero() {
            return Collision::none();
        }

        // Change referential
        let x = ugly.get_pos().x;
        let y = ugly.get_pos().y;
        let ux = drone.get_pos().x;
        let uy = drone.get_pos().y;

        let x2 = x - ux;
        let y2 = y - uy;
        let r2 = UGLY_EAT_RANGE + DRONE_HIT_RANGE;
        let vx2 = ugly.get_speed().x - drone.get_speed().x;
        let vy2 = ugly.get_speed().y - drone.get_speed().y;

        // Resolving: sqrt((x + t*vx)^2 + (y + t*vy)^2) = radius <=> t^2*(vx^2 + vy^2) + t*2*(x*vx + y*vy) + x^2 + y^2 - radius^2 = 0
        // at^2 + bt + c = 0;
        // a = vx^2 + vy^2
        // b = 2*(x*vx + y*vy)
        // c = x^2 + y^2 - radius^2 

        let a = vx2 * vx2 + vy2 * vy2;

        if a <= 0.0 {
            return Collision::none();
        }

        let b = 2.0 * (x2 * vx2 + y2 * vy2);
        let c = x2 * x2 + y2 * y2 - r2 * r2;
        let delta = b * b - 4.0 * a * c;

        if delta < 0.0 {
            return Collision::none();
        }

        let t = (-b - delta.sqrt()) / (2.0 * a);

        if t <= 0.0 {
            return Collision::none();
        }

        if t > 1.0 {
            return Collision::none();
        }
        Collision::new(t, ugly.clone(), drone.clone())
    }

    fn has_first_to_scan_bonus(player: &Player, scan: &Scan) -> bool {
        first_to_scan.get(scan).map_or(-1, |&val| val) == player.get_index()
    }

    fn has_first_to_scan_all_fish_of_type(player: &Player, fish_type: &FishType) -> bool {
        first_to_scan_all_fish_of_type.get(fish_type).map_or(-1, |&val| val) == player.get_index()
    }

    fn has_first_to_scan_all_fish_of_color(player: &Player, color: usize) -> bool {
        first_to_scan_all_fish_of_color.get(&color).map_or(-1, |&val| val) == player.get_index()
    }

    // Add other methods and properties here...
}
