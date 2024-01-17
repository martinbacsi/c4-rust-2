
use crate::{fish::*, drone::*, scan::*};

use std::collections::{HashSet};

pub struct Player {
    pub message: String,
    pub drones: Vec<Drone>,
    pub scans: HashSet<Scan>,
    pub visible_fishes: HashSet<Fish>,
    pub count_fish_saved: Vec<i32>,
    pub points: i32,
}

impl Player {
    pub fn new() -> Player {
        Player {
            message: String::new(),
            drones: Vec::new(),
            scans: HashSet::new(),
            visible_fishes: HashSet::new(),
            count_fish_saved: Vec::new(),
            points: 0,
        }
    }

    pub fn get_expected_output_lines(&self) -> usize {
        self.drones.len()
    }

    pub fn get_message(&self) -> &str {
        &self.message
    }

    pub fn reset(&mut self) {
        self.message.clear();
        for drone in &mut self.drones {
            drone.move_direction = None;
            drone.fishes_scanned_this_turn.clear();
            drone.did_report = false;
            drone.message.clear();
        }
    }
}
