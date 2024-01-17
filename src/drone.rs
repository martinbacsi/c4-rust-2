use std::collections::HashSet;

use crate::{player::*, vector::*, scan::*};

// Assuming you already have the Vector, Player, and Scan structs from the previous examples

#[derive(Debug)]
pub struct Drone {
    pub id: i32,
    pub owner: Player,
    pub pos: Vector,
    pub speed: Vector,
    pub light: i32,
    pub battery: i32,
    pub scans: Vec<Scan>,
    pub fishes_scanned_this_turn: HashSet<i32>,
    pub light_switch: bool,
    pub light_on: bool,
    pub dying: bool,
    pub dead: bool,
    pub did_report: bool,
    pub die_at: f64,
    pub message: String,
    pub max_turns_spent_with_scan: i32,
    pub turns_spent_with_scan: i32,
    pub max_y: i32,
}

impl Drone {
    fn new(x: f64, y: i32, id: i32, owner: Player) -> Drone {
        Drone {
            id,
            owner,
            pos: Vector::new(x, y as f64),
            speed: Vector::ZERO,
            light: 0,
            battery: 100, // Assuming initial battery value
            scans: Vec::new(),
            fishes_scanned_this_turn: HashSet::new(),
            light_switch: false,
            light_on: false,
            dying: false,
            dead: false,
            did_report: false,
            die_at: 0.0,
            message: String::new(),
            max_turns_spent_with_scan: 0,
            turns_spent_with_scan: 0,
            max_y: 0,
        }
    }

    pub fn is_engine_on(&self) -> bool {
        self.speed != Vector::ZERO
    }

    pub fn is_light_on(&self) -> bool {
        self.light_on
    }

    pub fn drain_battery(&mut self) {
        self.battery -= 1; // Assuming LIGHT_BATTERY_COST is 1
    }

    pub fn recharge_battery(&mut self) {
        if self.battery < 100 {
            self.battery += 1; // Assuming DRONE_BATTERY_REGEN is 1
        }

        if self.battery >= 100 {
            self.battery = 100;
        }
    }

    pub fn is_dead_or_dying(&self) -> bool {
        self.dying || self.dead
    }

    pub fn get_x(&self) -> f64 {
        self.pos.x
    }

    pub fn get_y(&self) -> f64 {
        self.pos.y
    }

    pub fn scan_slot_to_string(&self, i: usize) -> String {
        if let Some(scan) = self.scans.get(i) {
            scan.to_input_string()
        } else {
            "-1 -1".to_string()
        }
    }

    pub fn set_message(&mut self, message: String) {
        self.message = message;
        if let Some(trimmed_message) = self.message.get(..48) {
            self.message = trimmed_message.to_string();
        }
    }
}