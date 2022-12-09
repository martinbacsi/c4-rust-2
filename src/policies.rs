use core::panic;

use crate::connect4::Connect4;
use crate::nn::{Linear, ReLU, Activation, Softmax};
use crate::connect4::Game;

pub struct Connect4Net {
    pub l_1: Linear<63, 128>,
    pub l_2: Linear<128, 96>,
    pub l_3: Linear<96, 64>,
    pub l_4: Linear<64, 48>,
    pub l_5: Linear<48, 12>,
}

pub trait Policy<G: Game<N>, const N: usize> {
    fn eval(&mut self, game: &G) -> ([f32; N], [f32; 3]);
}

pub trait NNPolicy<G: Game<N>, const N: usize> {
    fn new() -> Self;
    //hát ez msot ide behardcodeolodott. ooopsie
    fn forward(&self, xs: &[f32; 63]) -> ([f32;N], [f32;3]);
}

impl NNPolicy<Connect4, { Connect4::MAX_NUM_ACTIONS }> for Connect4Net {
    fn new() -> Self {
        Self {
            l_1: Linear::default(),
            l_2: Linear::default(),
            l_3: Linear::default(),
            l_4: Linear::default(),
            l_5: Linear::default(),
        }
    }

    fn forward(&self, xs: &[f32; ((Connect4::DIMS[2] as usize) * (Connect4::DIMS[3] as usize))]) -> ([f32;Connect4::MAX_NUM_ACTIONS], [f32;3]) {
        let a = ReLU.apply_1d(&self.l_1.forward(&xs));
        /*for f in self.l_1.weight {
            for ff in f {
                println!("{}", ff);
            }
            //println!("{}", f);
        }*/
       
        let b = ReLU.apply_1d(&self.l_2.forward(&a));
        
        let c = ReLU.apply_1d(&self.l_3.forward(&b));
        let d = ReLU.apply_1d(&self.l_4.forward(&c));
        let e = &self.l_5.forward(&d);

        for ff in e {
            println!("{}", ff);
        }
        panic!("a");

        let mut policy_logits: [f32; 9] = [0f32; 9];
        let mut outcome_logits: [f32; 3]= [0f32; 3];
        policy_logits.copy_from_slice(&e[0..9]);
        outcome_logits.copy_from_slice(&e[9..12]);
        (policy_logits, outcome_logits)
    }
}

impl Policy<Connect4, 9> for Connect4Net {
    fn eval(&mut self, env: &Connect4) -> ([f32; Connect4::MAX_NUM_ACTIONS], [f32; 3]) {
        let xs = env.features();
        let (logits, value) = self.forward(&xs);
        (logits, Softmax.apply_1d (&value))
    }
}
