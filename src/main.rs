use std::process::exit;

use policies::{Policy, Connect4Net, NNPolicy};
use connect4::{Connect4, Game};
use base65536::{load_1d, load_2d, f16_to_f32, f32_to_bf16};
use weights::*;
mod connect4;
mod policies;
mod nn;
mod weights;
mod base65536;



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
    print!("alma");
}
