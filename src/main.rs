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

use visu::run_visu;



fn main() {
    run_visu();

    //run_ppo();
    //let model2 = tch::CModule::load("C:\\csb\\test.pt").expect("badge");
    //print!(model2);
    //panic!();
  
}