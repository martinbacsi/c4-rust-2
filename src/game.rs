use std::collections::{HashMap, HashSet};
use rand::prelude::*;

use crate::{ugly::*, fish::*, player::*, scan::*, vector::*, collision::*, entity::*, drone::*, closest::* };

// Assuming you already have the necessary structs and enums from previous translations




fn update_ugly_target(ugly: &mut Ugly, players: &[Player]) -> bool {
    let mut targetable_drones = players
        .iter()
        .flat_map(|p| p.drones.iter().filter(|drone| {
            drone.pos.in_range(&ugly.pos, if drone.is_light_on() { Game::LIGHT_SCAN_RANGE } else { Game::DARK_SCAN_RANGE })
                && !drone.is_dead_or_dying()
        }))
        .peekable();

    if targetable_drones.peek().is_some() {
        let closest_targets = get_closest_to(ugly.pos, &mut targetable_drones);
        ugly.target = closest_targets.get_mean_pos();
        // for drone in closest_targets.list.iter() {
        //     times_aggroed[drone.owner.get_index()] += 1;
        // }
        true
    } else {
        ugly.target = None;
        false
    }
}


fn snap_to_ugly_zone(ugly: &mut Ugly) {
    if ugly.pos.y > Game::HEIGHT - 1.0 {
        ugly.pos.y = Game::HEIGHT - 1.0;
    } else if ugly.pos.y < Game::UGLY_UPPER_Y_LIMIT {
        ugly.pos.y = Game::UGLY_UPPER_Y_LIMIT;
    }
}

fn snap_to_drone_zone(drone: &mut Drone) {
    if drone.pos.y > Game::HEIGHT - 1.0 {
        drone.pos = Vector::new(drone.pos.x, Game::HEIGHT - 1.0);
    } else if drone.pos.y < Game::DRONE_UPPER_Y_LIMIT as f64 {
        drone.pos = Vector::new(drone.pos.x, Game::DRONE_UPPER_Y_LIMIT as f64);
    }

    if drone.pos.x < 0.0 {
        drone.pos = Vector::new(0.0, drone.pos.y);
    } else if drone.pos.x >= Game::WIDTH {
        drone.pos = Vector::new( Game::WIDTH - 1.0, drone.pos.y);
    }
}

fn snap_to_fish_zone(fish: &mut Fish) {
    if fish.pos.y > (Game::HEIGHT - 1.0) as f64 {
        fish.pos = Vector::new(fish.pos.x, (Game::HEIGHT - 1.0) as f64);
    } else if fish.pos.y > fish.high_y as f64 {
        fish.pos = Vector::new(fish.pos.x, fish.high_y as f64);
    } else if fish.pos.y < fish.low_y as f64 {
        fish.pos = Vector::new(fish.pos.x, fish.low_y as f64);
    }
}


#[derive(Debug)]
pub struct Game {
    random: rand::prelude::ThreadRng,
    pub players: Vec<Player>,
    fishes: Vec<Fish>,
    uglies: Vec<Ugly>,
    first_to_scan: HashMap<Scan, i32>,
    first_to_scan_temp: HashMap<Scan, i32>,
    first_to_scan_all_fish_of_color: HashMap<i32, i32>,
    first_to_scan_all_fish_of_color_temp: HashMap<i32, i32>,
    first_to_scan_all_fish_of_type: HashMap<FishType, i32>,
    first_to_scan_all_fish_of_type_temp: HashMap<FishType, i32>,
    entity_count: i32,
    game_turn: i32,

    pub  chased_fish_count: [i32; 2],
    pub  times_aggroed: [i32; 2],
    pub  max_turns_spent_with_scan: [i32; 2],
    pub  max_y: [i32; 2],
    pub  turn_saved_fish: [[i32; 12]; 2],
    pub  drones_eaten: i32,
    pub  fish_scanned: i32
}

impl Game {
    pub const COLORS: [&'static str; 4] = ["pink", "yellow", "green", "blue"];
    pub const WIDTH: f64 = 10000.0;
    pub const HEIGHT: f64 = 10000.0;
    pub const UGLY_UPPER_Y_LIMIT: f64 = 2500.0;
    pub const DRONE_UPPER_Y_LIMIT: f64 = 0.0;
    pub const DRONE_START_Y: f64 = 500.0;
    pub const COLORS_PER_FISH: i32 = 4;
    pub const DRONE_MAX_BATTERY: i32 = 30;
    pub const LIGHT_BATTERY_COST: i32 = 5;
    pub const DRONE_BATTERY_REGEN: i32 = 1;
    pub const DRONE_MAX_SCANS: i32 = i32::MAX;
    pub const DARK_SCAN_RANGE: f64 = 800.0;
    pub const LIGHT_SCAN_RANGE: f64 = 2000.0;
    pub const UGLY_EAT_RANGE: i32 = 300;
    pub const DRONE_HIT_RANGE: i32 = 200;
    pub const FISH_HEARING_RANGE: f64 = (Game::DARK_SCAN_RANGE + Game::LIGHT_SCAN_RANGE) / 2.0;
    pub const DRONE_MOVE_SPEED: f64 = 600.0;
    pub const DRONE_SINK_SPEED: f64 = 300.0;
    pub const DRONE_EMERGENCY_SPEED: f64 = 300.0;
    pub const DRONE_MOVE_SPEED_LOSS_PER_SCAN: f64 = 0.0;
    pub const FISH_SWIM_SPEED: f64 = 200.0;
    pub const FISH_AVOID_RANGE: f64 = 600.0;
    pub const FISH_FLEE_SPEED: f64 = 400.0;
    pub const UGLY_ATTACK_SPEED: f64 = (Game::DRONE_MOVE_SPEED as f64 * 0.9) as f64;
    pub const UGLY_SEARCH_SPEED: f64 = (Game::UGLY_ATTACK_SPEED as f64 / 2.0) as f64;
    pub const FISH_X_SPAWN_LIMIT: f64 = 1000.0;
    pub const FISH_SPAWN_MIN_SEP: f64 = 1000.0;
    pub const ALLOW_EMOJI: bool = true;
    pub const CENTER: Vector = Vector {
        x: (Game::WIDTH - 1.0) as f64 / 2.0,
        y: (Game::HEIGHT - 1.0) as f64 / 2.0,
    };
    pub const MAX_TURNS: i32 = 201;


    pub const DRONES_PER_PLAYER: i32 = 2;
    pub const ENABLE_UGLIES: bool = false;
    pub const FISH_WILL_FLEE: bool = false;
    pub const FISH_WILL_MOVE: bool = true;
    pub const SIMPLE_SCANS: bool = true;

    

    pub fn new() -> Game {
        let mut ret = Game {
            random: rand::prelude::ThreadRng::default(),
            players: vec![Player::new(); 2],
            fishes: Vec::new(),
            uglies: Vec::new(),
            first_to_scan: HashMap::new(),
            first_to_scan_temp: HashMap::new(),
            first_to_scan_all_fish_of_color: HashMap::new(),
            first_to_scan_all_fish_of_color_temp: HashMap::new(),
            first_to_scan_all_fish_of_type: HashMap::new(),
            first_to_scan_all_fish_of_type_temp: HashMap::new(),
            entity_count: 0,
            game_turn: 0,

            chased_fish_count: [0; 2],
            times_aggroed: [0; 2],
            max_turns_spent_with_scan: [0; 2],
            max_y: [0; 2],
            turn_saved_fish: [[-1; 12], [-1; 12]],
            drones_eaten: 0,
            fish_scanned: 0,
        };
        ret.init();
        ret
    }

    pub fn init(&mut self) {
        self.entity_count = 0;
        self.random =rand::thread_rng();
        self.game_turn = 1;
        self.init_players();
        self.init_fish();
        self.init_uglies();

        for player in &mut self.players {
            if Game::SIMPLE_SCANS {
                player.visible_fishes = self.fishes.iter().map(|f| f.id).collect();
            }
        }
    }


    fn init_uglies(&mut self) {
        let ugly_count = if Game::ENABLE_UGLIES { 1 + self.random.gen::<i32>() % 3 } else { 0 };

        for _ in 0..ugly_count {
            let x = self.random.gen::<i32>() % (Game::WIDTH / 2.0) as i32;

            let y = (Game::HEIGHT / 2.0) as i32 + self.random.gen::<i32>() % (Game::HEIGHT / 2.0) as i32;
            for k in 0..2 {
                let mut ugly = Ugly::new(x as f64, y as f64, self.entity_count);
                if k == 1 {
                    ugly.pos = ugly.pos.hsymmetric(Game::CENTER.x);
                }

                self.uglies.push(ugly);
                self.entity_count += 1;
            }
        }
    }

    fn init_fish(&mut self) {

        for col in (0..Game::COLORS_PER_FISH).step_by(2) {
            for type_idx in 0..FishType::variants().len() {
                let mut position_found = false;
                let mut iterations = 0;
                let mut x = 0;
                let mut y = 0;

                let mut low_y = (Game::HEIGHT / 4.0) as i32;
                let mut high_y = Game::HEIGHT as i32;

                while !position_found {
                    x = self.random.gen::<i32>() % (Game::WIDTH - Game::FISH_X_SPAWN_LIMIT * 2.0) as i32 + Game::FISH_X_SPAWN_LIMIT as i32;
                    if type_idx == 0 {
                        y = (1.0 * Game::HEIGHT / 4.0) as i32 + Game::FISH_SPAWN_MIN_SEP as i32;
                        low_y = (1.0 * Game::HEIGHT / 4.0) as i32;
                        high_y = (2.0 * Game::HEIGHT / 4.0) as i32;
                    } else if type_idx == 1 {
                        y = (2.0 * Game::HEIGHT / 4.0) as i32 + Game::FISH_SPAWN_MIN_SEP as i32;
                        low_y = (2.0 * Game::HEIGHT / 4.0) as i32;
                        high_y = (3.0 * Game::HEIGHT / 4.0) as i32
                    } else {
                        y = (3.0 * Game::HEIGHT / 4.0) as i32 + Game::FISH_SPAWN_MIN_SEP as i32;
                        low_y = (3.0 * Game::HEIGHT / 4.0) as i32;
                        high_y = (4.0 * Game::HEIGHT / 4.0) as i32;
                    }
                    y += self.random.gen::<i32>() % (Game::HEIGHT / 4.0 - Game::FISH_SPAWN_MIN_SEP * 2.0) as i32;

                    let final_x = x;
                    let final_y = y;
                    let too_close = self.fishes.iter().any(|other| other.pos.in_range(&Vector::new(final_x as f64, final_y as f64), Game::FISH_SPAWN_MIN_SEP));
                    let too_close_to_center = (Game::CENTER.x - x as f64).abs() <= Game::FISH_SPAWN_MIN_SEP;
                    if !too_close && !too_close_to_center || iterations > 100 {
                        position_found = true;
                    }
                    iterations += 1;
                }
                let mut f = Fish::new(x as f64, y as f64, &FishType::variants()[type_idx], col, self.entity_count, low_y, high_y);

                let snapped = (self.random.gen::<i32>() % 7) as f64 * std::f64::consts::FRAC_PI_4;
                let direction = Vector::new(snapped.cos(), snapped.sin());

                if Game::FISH_WILL_MOVE {
                    f.speed = direction.mult(Game::FISH_SWIM_SPEED).round();
                }

               
                //TODO
                self.entity_count += 1;

                let other_pos = f.pos.hsymmetric(Game::CENTER.x);
                let other_speed = f.speed.hsymmetric(0.0);
                let mut o = Fish::new(other_pos.x, other_pos.y, &FishType::variants()[type_idx], col + 1, self.entity_count, f.low_y, f.high_y);
                o.speed = other_speed;
                self.fishes.push(o);
                self.fishes.push(f);
                self.entity_count += 1;
            }
        }
    }
        
    fn init_players(&mut self) {
        for i in 0..2 {
            self.players[i].index = i as i32;
        }
        let idxs = [0, 2, 1, 3];
        let mut idx_idx = 0;
        for _ in 0..Game::DRONES_PER_PLAYER {
            let x = Game::WIDTH / (Game::DRONES_PER_PLAYER as f64 * 2.0 + 1.0) * (idxs[idx_idx] as f64 + 1.0);
            idx_idx += 1;
            for player in &mut self.players {
                let mut drone = Drone::new(x as f64, Game::DRONE_START_Y as f64, self.entity_count, &player);

                if player.get_index() == 1 {
                    drone.pos = drone.pos.hsymmetric(Game::CENTER.x);
                }

                player.drones.push(drone);
                self.entity_count += 1;
            }
        }
    }

    pub fn reset_game_turn_data(&mut self) {
        for player in &mut self.players {
            player.reset();
        }
    }

    

    fn move_entities(&mut self) {
        // Move drones and handle collisions with uglies
        for player in self.players.iter_mut() {
            for drone in player.drones.iter_mut() {
                if drone.dead {
                    continue;
                }
            
                // NOTE: the collision code does not take into account the snap to map borders
                for ugly in  self.uglies.iter() {
                    let col = get_collision(drone, ugly);
                    if col >= 0.0 {
                        drone.dying = true;
                        drone.scans.clear();
                        drone.die_at = col;
                        //self.drones_eaten += 1;
                        // If two uglies hit the drone, let's just keep the first collision, it matters not.
                        break;
                    }
                }
            }
        }

        // Move drones and snap to drone zone
        for player in  self.players.iter_mut() {
            for drone in player.drones.iter_mut() {
                let speed = drone.get_speed();
                drone.pos = drone.pos.add(speed);
                snap_to_drone_zone(drone);
            }
        }

        // Move fishes and snap to fish zone
        for fish in  self.fishes.iter_mut() {
            fish.pos = fish.pos.add(fish.get_speed());
            snap_to_fish_zone(fish);
        }

        // Remove fishes that went out of bounds
        let fish_to_remove: Vec<i32> =  self.fishes
            .iter()
            .filter(|fish| fish.get_pos().x > Game::WIDTH - 1.0 || fish.get_pos().x < 0.0)
            .map(|f| f.id)
            .collect();


        self.fishes.retain(|fish| !fish_to_remove.iter().any(|f_id| *f_id == fish.id));

        // Reset fleeing information for remaining fishes
        for fish in self.fishes.iter_mut() {
            fish.fleeing_from_player = None;
        }

        // Move uglies and snap to ugly zone
        for ugly in self.uglies.iter_mut() {
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

    fn update_ugly_speeds(&mut self) {
        //TODO FIND BETTER SOLUTION
        let uglies_clone = self.uglies.clone(); 
        for ugly in self.uglies.iter_mut() {
            if let Some(target) = ugly.target {
                let attack_vec = Vector::from_points(ugly.pos, target);
                if attack_vec.length() > Game::UGLY_ATTACK_SPEED as f64 {
                    ugly.speed = attack_vec.normalize().mult(Game::UGLY_ATTACK_SPEED as f64).round();
                }
            } else {
                if ugly.speed.length() > Game::UGLY_SEARCH_SPEED as f64 {
                    ugly.speed = ugly.speed.normalize().mult(Game::UGLY_SEARCH_SPEED as f64).round();
                }

                if !ugly.speed.is_zero() {
                    let closest_uglies = get_closest_to(ugly.pos, uglies_clone.iter().filter(|u| u.id != ugly.id));
                    if !closest_uglies.list.is_empty() && closest_uglies.distance <= Game::FISH_AVOID_RANGE as f64 {
                        let avoid = closest_uglies.get_mean_pos().unwrap();
                        let avoid_dir = Vector::from_points(avoid, ugly.pos).normalize();
                        if !avoid_dir.is_zero() {
                            ugly.speed = avoid_dir.mult(Game::FISH_SWIM_SPEED as f64).round();
                        }
                    }
                }

                let next_pos = ugly.pos.add(ugly.speed);

                if (next_pos.x < 0.0 && next_pos.x < ugly.pos.x) || (next_pos.x > Game::WIDTH - 1.0 && next_pos.x > ugly.pos.x) {
                    ugly.speed = ugly.speed.hsymmetric(0.0);
                }

                if (next_pos.y < Game::UGLY_UPPER_Y_LIMIT as f64 && next_pos.y < ugly.pos.y)
                    || (next_pos.y > Game::HEIGHT - 1.0 && next_pos.y > ugly.pos.y)
                {
                    ugly.speed = ugly.speed.vsymmetric(0.0);
                }
            }
        }
    }

    fn update_ugly_targets(&mut self) {
        for ugly in &mut self.uglies {
            let found_target = update_ugly_target(ugly, &self.players);
            ugly.found_target = found_target;
        }
    }

    fn update_fish(&mut self) {
        let fishes_copy = self.fishes.clone(); 
        for fish in &mut self.fishes {
            fish.is_fleeing = false;

            let mut flee_from: Option<Vector> = None;
            if Game::FISH_WILL_FLEE {
                let closest_drones = get_closest_to(
                    fish.pos,
                    self.players.iter().flat_map(|p| p.drones.iter().filter(|d| d.is_engine_on() && !d.dead)),
                );

                if !closest_drones.list.is_empty() && closest_drones.distance <= Game::FISH_HEARING_RANGE as f64 {
                    flee_from = closest_drones.get_mean_pos();
                    let mut fleeing_from_player: Option<i32> = None;
                    for d in closest_drones.list.iter() {
                        let idx = d.owner as i32;
                        if fleeing_from_player.is_none() || fleeing_from_player.unwrap() == idx {
                            fleeing_from_player = Some(idx);
                        } else {
                            fleeing_from_player = Some(i32::MAX);
                        }
                    }
                    fish.fleeing_from_player = fleeing_from_player;
                }
            }

            if let Some(flee_from_vec) = flee_from {
                let flee_dir = Vector::from_points(flee_from_vec, fish.pos).normalize();
                let flee_vec = flee_dir.mult(Game::FISH_FLEE_SPEED as f64);
                fish.speed = flee_vec.round();
                fish.is_fleeing = true;
            } else {
                let swim_vec = fish.speed.normalize().mult(Game::FISH_SWIM_SPEED as f64);
                //TODO REMOVE CLONE
                
                let closest_fishes = get_closest_to(fish.pos, fishes_copy.iter().filter(|f| f.id != fish.id));

                if !closest_fishes.list.is_empty() && closest_fishes.distance <= Game::FISH_AVOID_RANGE as f64 {
                    let avoid = closest_fishes.get_mean_pos().unwrap();
                    let avoid_dir = Vector::from_points(avoid, fish.pos).normalize();
                    fish.speed = avoid_dir.mult(Game::FISH_SWIM_SPEED as f64);
                }

                let next_pos = fish.pos.add(fish.speed);

                if (next_pos.x < 0.0 && next_pos.x < fish.pos.x)
                    || (next_pos.x > Game::WIDTH - 1.0 && next_pos.x > fish.pos.x)
                {
                    fish.speed = fish.speed.hsymmetric(0.0);
                }

                let y_highest = f64::min(Game::HEIGHT as f64 - 1.0, fish.high_y as f64);

                if (next_pos.y < fish.low_y as f64 && next_pos.y < fish.pos.y)
                    || (next_pos.y > y_highest && next_pos.y > fish.pos.y)
                {
                    fish.speed = fish.speed.vsymmetric(0.0);
                }
                fish.speed = fish.speed.epsilon_round().round();
            }
        }
    }



    fn update_drones(&mut self) {
        for player in &mut self.players{
            for drone in player.drones.iter_mut() {
                let move_speed =
                    (Game::DRONE_MOVE_SPEED - Game::DRONE_MOVE_SPEED * Game::DRONE_MOVE_SPEED_LOSS_PER_SCAN * drone.scans.len() as f64) as f64;

                if drone.dead {
                    let float_vec = Vector::new(0.0, -1.0).mult(Game::DRONE_EMERGENCY_SPEED as f64);
                    drone.speed = float_vec;
                } else if let Some(move_pos) = &drone.move_command {
                    let move_vec = Vector::from_points(drone.pos, *move_pos);

                    if move_vec.length() > move_speed {
                        drone.speed = move_vec.normalize().mult(move_speed).round();
                    } else {
                        drone.speed = move_vec.round();
                    }
                } else if drone.pos.y < Game::HEIGHT - 1.0 {
                    let sink_vec = Vector::new(0.0, 1.0).mult(Game::DRONE_SINK_SPEED as f64);
                    drone.speed = sink_vec;
                }
            }
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
        }

        self.game_turn += 1;
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
                    player.visible_fishes.insert(fish.id);

                    if drone.scans.len() < Game::DRONE_MAX_SCANS as usize {
                        let scan = Scan::new_from_fish(&fish);
                        if !player.scans.contains(&scan) {
                            if !drone.scans.contains(&scan) {
                                drone.scans.insert(scan);
                                drone.fishes_scanned_this_turn.insert(fish.id);
                            }
                        }
                    }
                }

                if Game::SIMPLE_SCANS {
                    player.visible_fishes = self.fishes.iter().map(|f| f.id).collect();
                }
            }
        }
    }

    fn apply_scans_for_report(&mut self, player_index: usize, drone_index: usize) -> bool {
        let player = &mut self.players[player_index];
        for scan in &player.scans {
            if !self.first_to_scan.contains_key(scan) {
                if self.first_to_scan_temp.contains_key(scan) {
                    // Opponent also completed this report this turn, nobody gets the bonus
                    self.first_to_scan_temp.remove(scan);
                } else {
                    self.first_to_scan_temp.insert(scan.clone(), player_index as i32);
                }
            }
            let fish_index = scan.color * 3 + scan.fish_type as i32;
            self.turn_saved_fish[player_index as usize][fish_index as usize] = self.game_turn;
            self.fish_scanned += 1;
        }
        if ! player.drones[drone_index].scans.is_empty() {
            player.count_fish_saved.push( player.drones[drone_index].scans.len() as i32);
        }
        let size_before = player.scans.len();
        player.scans.extend( player.drones[drone_index].scans.drain());

        // TODO NICER WAY
        let scans_clone = player.drones[drone_index].scans.clone();
        for other in player.drones.iter_mut() {
            if drone_index != other.id as usize {
                other.scans.retain(|s| !scans_clone.contains(s));
            }
        }
        size_before < player.scans.len()
    }

    fn detect_first_to_combo_bonuses(&mut self, player_index: usize) {
        for &fish_type in FishType::variants().iter() {
            if !self.first_to_scan_all_fish_of_type.contains_key(&fish_type) {
                if self.player_scanned_all_fish_of_type(player_index, fish_type) {
                    if self.first_to_scan_all_fish_of_type_temp.contains_key(&fish_type) {
                        // Opponent also completed this report this turn, nobody gets the bonus
                        self.first_to_scan_all_fish_of_type_temp.remove(&fish_type);
                    } else {
                        self.first_to_scan_all_fish_of_type_temp.insert(fish_type, player_index as i32);
                    }
                }
            }
        }

        for color in 0..Game::COLORS_PER_FISH {
            if !self.first_to_scan_all_fish_of_color.contains_key(&color) {
                if self.player_scanned_all_fish_of_color(player_index, color) {
                    if self.first_to_scan_all_fish_of_color_temp.contains_key(&color) {
                        // Opponent also completed this report this turn, nobody gets the bonus
                        self.first_to_scan_all_fish_of_color_temp.remove(&color);
                    } else {
                        self.first_to_scan_all_fish_of_color_temp.insert(color, player_index as i32);
                    }
                }
            }
        }
    }

    fn do_report(&mut self) {
         for player_index in 0..self.players.len() { 
            let mut points_scored = false;
            for drone_index in 0..self.players[player_index].drones.len() {
                if self.players[player_index].drones[drone_index].is_dead_or_dying() {
                    continue;
                }
                if Game::SIMPLE_SCANS || (!self.players[player_index].drones[drone_index].scans.is_empty() && self.players[player_index].drones[drone_index].pos.y <= Game::DRONE_START_Y as f64) {
                    let drone_scored = self.apply_scans_for_report(player_index, drone_index);
                    points_scored |= drone_scored;
                    if drone_scored {
                        self.players[player_index].drones[drone_index].did_report = true;
                    }
                }
            }
            self.detect_first_to_combo_bonuses(player_index);
        }

        self.persist_first_to_scan_bonuses();
        for i in 0..2 {
            self.players[i].points = self.compute_player_score(i);
        }
    }

    fn persist_first_to_scan_bonuses(&mut self) {
        let mut player_scans_map: HashMap<i32, Vec<Scan>> = HashMap::new();

        for (scan, &player_index) in &self.first_to_scan_temp {
            self.first_to_scan.entry(scan.clone()).or_insert(player_index);

            player_scans_map.entry(player_index)
                .or_insert_with(|| Vec::new())
                .push(scan.clone());
        }

        for (player_name, player_scans) in player_scans_map {
            let summary_string = player_scans.iter()
                .map(|scan| scan.fish_id.to_string())
                .collect::<Vec<String>>()
                .join(", ");
        
            if player_scans.len() == 1 {
                eprintln!("{} was the first to save the scan of creature {}", player_name, summary_string);
            } else {
                eprintln!(
                        "{} was the first to save the scans of {} creatures: {}", 
                        player_name, 
                        player_scans.len(), 
                        summary_string);
            }
        }

        for (fish_type, &player_index) in &self.first_to_scan_all_fish_of_type_temp {
            self.first_to_scan_all_fish_of_type.entry(*fish_type).or_insert(player_index);
            eprintln!("player{} scanned all of fish type{}", player_index, *fish_type as usize);
        }

        for (color, &player_index) in &self.first_to_scan_all_fish_of_color_temp {
            self.first_to_scan_all_fish_of_color.entry(*color).or_insert(player_index);
            eprintln!("player{} scanned all of fish color{}", player_index, Game::COLORS[*color as usize]);
        }

        self.first_to_scan_temp.clear();
        self.first_to_scan_all_fish_of_color_temp.clear();
        self.first_to_scan_all_fish_of_type_temp.clear();
    }

    fn player_scanned(&self,  player_index: usize, fish: &Fish) -> bool {
        self.player_scanned_scan(player_index, &Scan::new_from_fish(fish))
    }

    fn player_scanned_scan(&self, player_index: usize, scan: &Scan) -> bool {
        self.players[player_index].scans.contains(scan)
    }

    fn has_scanned_all_remaining_fish(&self,  player_index: usize) -> bool {
        self.fishes.iter().all(|fish| self.player_scanned(player_index, fish))
    }

    fn has_fish_escaped(&self, scan: &Scan) -> bool {
        !self.fishes.iter().any(|fish| fish.color == scan.color && fish.fish_type == scan.fish_type)
    }

    fn is_fish_scanned_by_player_drone(&self, scan: &Scan,  player_index: usize) -> bool {
        self.players[player_index].drones.iter().any(|drone| drone.scans.contains(scan))
    }

    fn is_type_combo_still_possible(&self, player_index: usize, fish_type: &FishType) -> bool {
        if self.player_scanned_all_fish_of_type(player_index, *fish_type) {
            return false;
        }

        for color in 0..Game::COLORS_PER_FISH {
            let scan = Scan::new_from_type_color(*fish_type, color);

            if self.has_fish_escaped(&scan) && !self.is_fish_scanned_by_player_drone(&scan, player_index) && !self.player_scanned_scan(player_index, &scan) {
                return false;
            }
        }
        true
    }

    fn is_color_combo_still_possible(&self, player_index: usize, color: i32) -> bool {
        if self.player_scanned_all_fish_of_color(player_index, color) {
            return false;
        }

        for fish_type in FishType::variants() {
            let scan = Scan::new_from_type_color(*fish_type, color);
            if self.has_fish_escaped(&scan) && !self.is_fish_scanned_by_player_drone(&scan, player_index) && !self.player_scanned_scan(player_index, &scan) {
                return false;
            }
        }
        true
    }

    fn compute_max_player_score(&self,  player_index: usize) -> i32 {
        let mut total = self.compute_player_score(player_index);
        let p2 = &self.players[1 - player_index];

        for color in 0..Game::COLORS_PER_FISH {
            for fish_type in FishType::variants() {
                let scan = Scan::new_from_type_color(*fish_type, color);
                if !self.player_scanned_scan(player_index, &scan) {
                    if self.is_fish_scanned_by_player_drone(&scan, player_index) || !self.has_fish_escaped(&scan) {
                        total += *fish_type as i32 + 1;
                        if self.first_to_scan.get(&scan).map_or(true, |&val| val == -1) {
                            total += *fish_type as i32 + 1;
                        }
                    }
                }
            }
        }

        for fish_type in FishType::variants() {
            if self.is_type_combo_still_possible(player_index, &fish_type) {
                total += Game::COLORS_PER_FISH as i32;
                if self.first_to_scan_all_fish_of_type.get(&fish_type).map_or(true, |&val| val != p2.get_index()) {
                    total += Game::COLORS_PER_FISH as i32;
                }
            }
        }

        for color in 0..Game::COLORS_PER_FISH {
            if self.is_color_combo_still_possible(player_index, color) {
                total += FishType::variants().len() as i32;
                if self.first_to_scan_all_fish_of_color.get(&color).map_or(true, |&val| val != p2.get_index()) {
                    total += FishType::variants().len() as i32;
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
                || self.compute_max_player_score(0) < self.players[1].points
                || self.compute_max_player_score(1) < self.players[0].points
        }
    }

    fn both_players_have_scanned_all_remaining_fish(&self) -> bool {
        self.players.iter().all(|player| self.has_scanned_all_remaining_fish(player.index as usize))
    }

    fn player_scanned_all_fish_of_color(&self, player_index: usize, color: i32) -> bool {
        FishType::variants().iter().all(|&fish_type| self.player_scanned_scan(player_index, &Scan::new_from_type_color(fish_type, color)))
    }

    fn compute_player_score(&self, player_index: usize) -> i32 {
        let mut total = 0;
        for scan in &self.players[player_index].scans {
            total += scan.fish_type as i32 + 1;
            if self.first_to_scan.get(scan).map_or(false, |&val| val == self.players[player_index].get_index()) {
                total += scan.fish_type as i32 + 1;
            }
        }

        for fish_type in FishType::variants() {
            if self.player_scanned_all_fish_of_type(self.players[player_index].index as usize, *fish_type) {
                total += Game::COLORS_PER_FISH as i32;
            }
            if self.first_to_scan_all_fish_of_type.get(&fish_type).map_or(false, |&val| val == self.players[player_index].get_index()) {
                total += Game::COLORS_PER_FISH as i32;
            }
        }

        for color in 0..Game::COLORS_PER_FISH {
            if self.player_scanned_all_fish_of_color(player_index, color) {
                total += FishType::variants().len() as i32;
            }
            if self.first_to_scan_all_fish_of_color.get(&color).map_or(false, |&val| val == player_index as i32) {
                total += FishType::variants().len() as i32;
            }
        }

        total
    }

    fn player_scanned_all_fish_of_type(&self, player_index: usize, fish_type: FishType) -> bool {
        (0..Game::COLORS_PER_FISH).all(|color| self.player_scanned_scan(player_index, &Scan::new_from_type_color(fish_type, color)))
    }

    fn compute_score_on_end(&mut self) {
         for player_index in 0..self.players.len() {
            for drone_index in 0..self.players[player_index].drones.len() {
                self.apply_scans_for_report(player_index, drone_index);
            }
        
            self.detect_first_to_combo_bonuses(player_index);
        }

        self.persist_first_to_scan_bonuses();

        for i in 0..2 {

            let score = self.compute_player_score(i);
            self.players[i].set_score(score);
            self.players[i].points = score;

        }
    }

    fn has_first_to_scan_bonus(&self, player: &Player, scan: &Scan) -> bool {
        self.first_to_scan.get(scan).map_or(-1, |&val| val) == player.get_index()
    }

    fn has_first_to_scan_all_fish_of_type(&self,player: &Player, fish_type: &FishType) -> bool {
        self.first_to_scan_all_fish_of_type.get(fish_type).map_or(-1, |&val| val) == player.get_index()
    }

    fn has_first_to_scan_all_fish_of_color(&self,player: &Player, color: i32) -> bool {
        self.first_to_scan_all_fish_of_color.get(&color).map_or(-1, |&val| val) == player.get_index()
    }

    // Add other methods and properties here...
}
