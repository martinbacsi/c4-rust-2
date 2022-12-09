use std::process::exit;

use policies::{Policy, Connect4Net, NNPolicy};
use connect4::{Connect4, Game};
use base65536::{load_1d, load_2d, f16_to_f32, f32_to_bf16};
use mcts::MCTS;
use weights::*;
use config::*;

mod connect4;
mod policies;
mod nn;
mod weights;
mod base65536;
mod mcts;
mod config;


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
    let c4 = Connect4::new();
    let (a, b) = policy.eval(&c4);

    for ff in a {
        println!("{}", ff);
    }

    let mcts_cfg = MCTSConfig {
        exploration: Exploration::PolynomialUct { c: 3.0 }, // type of exploration to use (e.g. PUCT or UCT)
        solve: true,                   // use MCTS Solver extension to solve nodes
        correct_values_on_solve: true, // if node is solved, adjust previously backprop'd values
        select_solved_nodes: true,     // select nodes that are solved
        auto_extend: true,             // visit nodes until a node with > 1 child is reached
        fpu: Fpu::ParentQ,
        root_policy_noise: PolicyNoise::None,
    };

    let mut mcts =
            MCTS::with_capacity(2000, mcts_cfg, &mut policy, c4.clone());

        // explore
        mcts.explore_n(1000);


    let a = mcts.best_action(ActionSelection::NumVisits);
    let b: usize = a.into();
    print!("{}", b);
}
