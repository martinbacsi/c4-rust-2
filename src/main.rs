use std::process::exit;

use policies::{Policy, Connect4Net, NNPolicy};
use connect4::{Connect4, Game, Column};
use base65536::{load_1d, load_2d, f16_to_f32, f32_to_bf16};
use mcts::MCTS;
use weights::*;
use config::*;
use std::io;
use std::time::Duration;
use std::{collections::HashMap, mem, time::Instant};
//use crate::connect4::Column;

mod connect4;
mod policies;
mod nn;
mod weights;
mod base65536;
mod mcts;
mod config;

macro_rules! parse_input {
    ($x:expr, $t:ident) => {
        $x.trim().parse::<$t>().unwrap()
    };
}

fn main() {
    
    //println!("{}", f16_to_f32( f32_to_bf16(512321f32) ));
    //exit(2);
    let mut policy = Connect4Net::new();
    load_2d(&mut policy.l_1.weight, String::from(PARAMETERS[0]));
    load_1d(&mut policy.l_1.bias, String::from(PARAMETERS[1]));
    load_2d(&mut policy.l_2.weight, String::from(PARAMETERS[2]));
    load_1d(&mut policy.l_2.bias, String::from(PARAMETERS[3]));
    load_2d(&mut policy.l_3.weight, String::from(PARAMETERS[4]));
    load_1d(&mut policy.l_3.bias, String::from(PARAMETERS[5]));
    load_2d(&mut policy.l_4.weight, String::from(PARAMETERS[6]));
    load_1d(&mut policy.l_4.bias, String::from(PARAMETERS[7]));
    load_2d(&mut policy.l_5.weight, String::from(PARAMETERS[8]));
    load_1d(&mut policy.l_5.bias, String::from(PARAMETERS[9]));
    let mut c4 = Connect4::new();
    //let (a, b) = policy.eval(&c4);

    //for ff in a {
    //    println!("{}", ff);
    //}

    let mcts_cfg = MCTSConfig {
        exploration: Exploration::PolynomialUct { c: 3.0 }, // type of exploration to use (e.g. PUCT or UCT)
        solve: true,                   // use MCTS Solver extension to solve nodes
        correct_values_on_solve: true, // if node is solved, adjust previously backprop'd values
        select_solved_nodes: true,     // select nodes that are solved
        auto_extend: true,             // visit nodes until a node with > 1 child is reached
        fpu: Fpu::ParentQ,
        root_policy_noise: PolicyNoise::None,
    };



    let mut endt;
    let mut input_line = String::new();
    io::stdin().read_line(&mut input_line).unwrap();
    let mut my_last: i32 = -1;
    for i in 0..65 {
        io::stdin().read_line(&mut input_line).unwrap();
        for _ in 0..7 as usize {
            io::stdin().read_line(&mut input_line).unwrap();
        }
        input_line.clear();
        io::stdin().read_line(&mut input_line).unwrap();
        let num_valid_actions = parse_input!(input_line, i32);
        for _ in 0..num_valid_actions as usize {
            io::stdin().read_line(&mut input_line).unwrap();
        }
        input_line.clear();
        io::stdin().read_line(&mut input_line).unwrap();
        if i == 0 {
            endt = Instant::now() + Duration::from_millis(900);
        } else {
            endt = Instant::now() + Duration::from_millis(100);
        }
        if my_last != -1 {
            c4.step(&Column::from(my_last as usize));
            //self.update_with_action(my_last as u8);
        }

        let mut hard_coded: i32 = -1;
        if input_line != "STEAL" {
            let opp_action = parse_input!(input_line, i32);
            if opp_action >= 0 {
                c4.step(&Column::from(opp_action as usize));
            }
            if opp_action == -1 {
                hard_coded = 1;
                c4.step(&Column::from(hard_coded as usize));
                my_last = -1;
            } else if i == 0 {
                hard_coded = -2;
            }
        }
        let mut mcts =
        MCTS::with_capacity(20000, mcts_cfg, &mut policy, c4.clone());

    // explore
        mcts.explore_until(endt);
       
        let mut a = mcts.best_action(ActionSelection::NumVisits);
        //c4.step(&a) ;
        if hard_coded >= 0 {
            a = Column::from(hard_coded as usize);
        } else {
            if hard_coded == -2 {
                my_last = -1;
                println!("STEAL");
                continue;
            }
            let aa: usize = a.into();
            my_last = aa as i32;
        }
        let aa: usize = a.into();
        //c4.print();
        println!("{}", aa);

    }


    //while true {
    //    let mut mcts =
    //            MCTS::with_capacity(2000, mcts_cfg, &mut policy, c4.clone());

            // explore
    //        mcts.explore_n(5000);

    //    let a = mcts.best_action(ActionSelection::NumVisits);
    //    c4.step(&a) ;
    //    c4.print();
    //}
    
 
}
