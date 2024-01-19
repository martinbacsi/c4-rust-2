use core::panic;


mod csb;
mod visu;
mod game;
mod ugly;
mod entity;
mod scan;
mod vector;
mod fish;
mod drone;
mod collision;
mod player;
mod closest;
mod referee;
mod ppo;
mod xorshift;

use xorshift::*;
use visu::run_visu;
use game::Game;

use std::fs::File;
use std::io::{self, BufRead};

macro_rules! parse_input {
    ($x:expr, $t:ident) => ($x.trim().parse::<$t>().unwrap())
}

fn read_lines_from_file(file_path: &str) -> Vec<String> {
    let file = File::open(file_path).expect("Unable to open file");
    io::BufReader::new(file).lines().map(|line| line.unwrap()).collect()
}



fn main() {
    let mut xor = xorshift::xorshift::new(69);
    for i in 0..400 {
        eprintln!("{}", xor.next_in_range(420)) ;
    }
    //panic!();

    let mut g = Game::new();
    let lines = read_lines_from_file("C://cg23_java//example.txt");
    eprintln!("{}", lines[0]);
    let creature_count = parse_input!(lines[0], i32);
    let mut line_index = 1;

    for i in 0..creature_count as usize {
        let inputs = lines[line_index].split_whitespace().collect::<Vec<_>>();
        let creature_id = parse_input!(inputs[0], i32);
        let color = parse_input!(inputs[1], i32);
        let _type = parse_input!(inputs[2], i32);
        line_index += 1;
    }
    
    for i in 0..201 {
        eprintln!("turn{}", i);
        let my_score = parse_input!(lines[line_index], i32);
        line_index += 1;
        let foe_score = parse_input!(lines[line_index], i32);
        line_index += 1;
        let my_scan_count = parse_input!(lines[line_index], i32);
        line_index += 1;

        for i in 0..my_scan_count as usize {
            let creature_id = parse_input!(lines[line_index], i32);
            line_index += 1;
        }

        let foe_scan_count = parse_input!(lines[line_index], i32);
        line_index += 1;

        for i in 0..foe_scan_count as usize {
            let creature_id = parse_input!(lines[line_index], i32);
            line_index += 1;
        }

        let my_drone_count = parse_input!(lines[line_index], i32);
        line_index += 1;

        for i in 0..my_drone_count as usize {
            let inputs = lines[line_index].split_whitespace().collect::<Vec<_>>();
            let drone_id = parse_input!(inputs[0], i32);
            let drone_x = parse_input!(inputs[1], i32);
            let drone_y = parse_input!(inputs[2], i32);
            let emergency = parse_input!(inputs[3], i32);
            let battery = parse_input!(inputs[4], i32);
            line_index += 1;
            assert_eq!(g.players[0].drones[i].get_x() as i32, drone_x, "drone {} kaka", drone_id);
            assert_eq!(g.players[0].drones[i].get_y() as i32, drone_y, "drone {} kaka", drone_id);
        }

        let foe_drone_count = parse_input!(lines[line_index], i32);
        line_index += 1;

        for i in 0..foe_drone_count as usize {
            let inputs = lines[line_index].split_whitespace().collect::<Vec<_>>();
            let drone_id = parse_input!(inputs[0], i32);
            let drone_x = parse_input!(inputs[1], i32);
            let drone_y = parse_input!(inputs[2], i32);
            let emergency = parse_input!(inputs[3], i32);
            let battery = parse_input!(inputs[4], i32);
            line_index += 1;
        }

        let drone_scan_count = parse_input!(lines[line_index], i32);
        line_index += 1;

        for i in 0..drone_scan_count as usize {
            let inputs = lines[line_index].split_whitespace().collect::<Vec<_>>();
            let drone_id = parse_input!(inputs[0], i32);
            let creature_id = parse_input!(inputs[1], i32);
            line_index += 1;
        }

        let visible_creature_count = parse_input!(lines[line_index], i32);
        line_index += 1;

        for i in 0..visible_creature_count as usize {
            let inputs = lines[line_index].split_whitespace().collect::<Vec<_>>();
            let creature_id = parse_input!(inputs[0], i32);
            let creature_x = parse_input!(inputs[1], i32);
            let creature_y = parse_input!(inputs[2], i32);
            let creature_vx = parse_input!(inputs[3], i32);
            let creature_vy = parse_input!(inputs[4], i32);
            line_index += 1;
            let f = g.fishes.iter().find(|f| f.id == creature_id).unwrap();
            if f.id == 9 {
                 eprintln!("..................................");
                 eprintln!("speed: {} {}",creature_vx, creature_vy);
                 eprintln!("pos: {} {}", creature_x,creature_y);

                 eprintln!("speed: {} {}", f.speed.x, f.speed.y);
                 eprintln!("pos: {} {}", f.get_x(), f.get_y());
                 eprintln!("..................................");
            }

           
            assert_eq!(f.get_x() as i32, creature_x,  "fish {} kaka", creature_id);
            assert_eq!(f.get_y() as i32, creature_y,  "fish {} kaka", creature_id);
        }

        let radar_blip_count = parse_input!(lines[line_index], i32);
        line_index += 1;

        for i in 0..radar_blip_count as usize {
            let inputs = lines[line_index].split_whitespace().collect::<Vec<_>>();
            let drone_id = parse_input!(inputs[0], i32);
            let creature_id = parse_input!(inputs[1], i32);
            let radar = inputs[2].trim().to_string();
            line_index += 1;
        }
        g.perform_game_update(0);
    }
    

    //run_visu();

    //run_ppo();
    //let model2 = tch::CModule::load("C:\\csb\\test.pt").expect("badge");
    //print!(model2);
    //panic!();
  
}