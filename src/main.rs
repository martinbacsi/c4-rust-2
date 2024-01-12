use core::panic;
use tch::{nn, Device, Tensor, nn::OptimizerConfig, nn::Adam, nn::VarStore, nn::Optimizer};

mod csb;
mod visu;

use visu::Visualizer;

use crate::csb::CSB_Game;
use rand::prelude::*;

const LEARNING_RATE: f64 = 0.00001;
const GAMMA: f64 = 0.98;
const LAMBDA: f64 = 0.95;
const EPS_CLIP: f32 = 0.1;
const K_EPOCH: usize = 3;

struct TrainItem {
    state: [f32; 10],
    action: f32,
    reward: f32,
    state_prime: [f32; 10],
    done: f32,
    probability: f32,
}

struct PPO {
    fc0: nn::Linear,
    fc1: nn::Linear,
    fc2: nn::Linear,
    fc3: nn::Linear,
    fc4: nn::Linear,
    fc_pi: nn::Linear,
    fc_v: nn::Linear,
    optimizer: nn::Optimizer<nn::Adam>,
    data: Vec<TrainItem>,
    vs: VarStore
}


impl PPO {
    fn new() -> PPO {
        let mut vs = VarStore::new(tch::Device::Cpu);
        let root = &vs.root();
        let fc0 = nn::linear(root / "fc0", 10, 16, Default::default());
        let fc1 = nn::linear(root / "fc1", 16, 32, Default::default());
        let fc2 = nn::linear(root / "fc2", 32,32, Default::default());
        let fc3 = nn::linear(root / "fc3", 32, 64, Default::default());
        let fc4 = nn::linear(root / "fc4", 64, 64, Default::default());
        let fc_pi = nn::linear(root / "fc_pi", 64, 6, Default::default());
        let fc_v = nn::linear(root / "fc_v", 64, 1, Default::default());

        let optimizer = nn::Adam::default().build(&vs, LEARNING_RATE).unwrap();
     
        vs.load("model_0.ot").expect("failed to load");
        
        PPO { fc0, fc1, fc2, fc3, fc4, fc_pi, fc_v, optimizer, data: Vec::new(), vs }
    }

    fn pi(&self, x: &Tensor) -> Tensor {       
        let x = x 
           .flat_view()
           .apply(&self.fc0)
           .relu()
           .apply(&self.fc1)
           .relu()
           .apply(&self.fc2)
           .relu()
           .apply(&self.fc3)
           .relu()
           .apply(&self.fc4)
           .relu();
       let pi = x.apply(&self.fc_pi).softmax(-1, tch::Kind::Float);
       pi
   } 

    fn v(&self, x: &Tensor) -> Tensor{       
         let x = x 
            .flat_view()
            .apply(&self.fc0)
            .relu()
            .apply(&self.fc1)
            .relu()
            .apply(&self.fc2)
            .relu()
            .apply(&self.fc3)
            .relu()
            .apply(&self.fc4)
            .relu();
        let v =  x.apply(&self.fc_v);
        v
    }

    fn put_data(&mut self, transition: TrainItem) {
        self.data.push(transition);
    }

    fn make_batch(&mut self) -> (Tensor, Tensor, Tensor, Tensor, Tensor, Tensor) {
        let data_len = self.data.len() as i64;
        let s = Tensor::zeros(&[data_len, 10], (tch::Kind::Float, tch::Device::Cpu));
        let a = Tensor::zeros(&[data_len, 1], (tch::Kind::Int64, tch::Device::Cpu));
        let r = Tensor::zeros(&[data_len, 1], (tch::Kind::Float, tch::Device::Cpu));
        let s_prime = Tensor::zeros(&[data_len, 10], (tch::Kind::Float, tch::Device::Cpu));
        let done_mask = Tensor::zeros(&[data_len, 1], (tch::Kind::Float, tch::Device::Cpu));
        let prob_a = Tensor::zeros(&[data_len, 1], (tch::Kind::Float, tch::Device::Cpu));
        
       
        unsafe {
            let s_data =  s.data_ptr() as *mut f32;
            let a_data =  a.data_ptr() as *mut i64;
            let r_data =  r.data_ptr() as *mut f32;
            let s_prime_data =  s_prime.data_ptr() as *mut f32;
            let done_mask_data =  done_mask.data_ptr() as *mut f32;
            let prob_a_data =  prob_a.data_ptr() as *mut f32;

            for i in 0..data_len as usize {
                let v = &self.data[i as usize];
                for j in 0..10 as usize {
                    let id = (i * 10 + j) as isize;
                    *s_data.offset(id)  = v.state[j] as f32;
                    *s_prime_data.offset(id) = v.state_prime[j] as f32;
                } 
                
                *a_data.offset(i as isize) = v.action as i64;
                *r_data.offset(i as isize) = v.reward as f32;
                *done_mask_data.offset(i as isize) = v.done as f32;
                *prob_a_data.offset(i as isize) = v.probability as f32;
            }
         
            self.data.clear();
        }
        (s, a, r, s_prime, done_mask, prob_a)
    }

    fn train_net(&mut self) {
        let (s, a, r, s_prime, done_mask, prob_a) = self.make_batch();
        
        for _ in 0..K_EPOCH {         
            let td_target = &r + &self.v(&s_prime) * done_mask.detach() * GAMMA;
           
            let delta = (&td_target - &self.v(&s)).detach();
           
            //(&r + &self.v(&s_prime)).print();
            let mut advantage: f32 = 0.0; 
            let mut advantage_list = Vec::new();

            unsafe {
                let numel = delta.numel();
                let d_ptr = delta.data_ptr() as *const f32;
                for i in 0..numel {
                    advantage = ((GAMMA * LAMBDA) as f32) * advantage + *d_ptr.offset((numel -  i - 1) as isize);
                    advantage_list.push(advantage);
                }
            }
            advantage_list.reverse();
            // Initialize advantage.
            let advantage_tensor = Tensor::of_slice(&advantage_list).view([-1]);
            
            //advantage_tensor.print();
            //panic!();
            
            let pi = &self.pi(&s);
            let pi_a = pi.gather(1, &a, false);    
            
            let ratio = (&pi_a.log() - &prob_a.log()).exp();

            let surr1 = &ratio * &advantage_tensor;        
            let surr2 = ratio.clamp((1.0 - EPS_CLIP) as f64, (1.0 + EPS_CLIP) as f64) * &advantage_tensor;
            let l1 =  &self.v(&s).smooth_l1_loss(&td_target.detach(), tch::Reduction::Mean, 1.0);
            let loss = (-surr1.min1(&surr2) + l1).mean(tch::Kind::Float);
            self.optimizer.zero_grad();
            loss.backward();
            self.optimizer.step();
            self.vs.save("model_0.ot");
        }
    }
}

fn categorical_sample(probs: &[f32; 6]) -> usize {
    let mut rng = rand::thread_rng();
    let rand_value: f32 = rng.gen_range(0.0..1.0); 
    let mut cumulative_prob = 0.0;
    for (i, &prob) in probs.iter().enumerate() {
        cumulative_prob += prob;
        if rand_value <= cumulative_prob {
            return i;
        }
    }
    panic!();
}

fn main() {
    //let model2 = tch::CModule::load("C:\\csb\\test.pt").expect("badge");
    //print!(model2);
    //panic!();
    let mut model = PPO::new();
    let mut probs: [f32; 6] = [0.0; 6];
    let mut scores: Vec<f64> = Vec::new();
    let mut visu = Visualizer::new(40., "puni");

    for n_epi in 0..100000 {
        let mut score = 0.0;
        let mut env = CSB_Game::new();
        let mut done = false;
        let mut s = env.encode();
       
        while !done {
            if n_epi % 100 == 0 {
                visu.draw(&env);
                visu.update();
            }
            
            let pi = model.pi(&Tensor::of_slice(&s).view([1, 10]));
         
            unsafe {
                let pi_ptr = pi.data_ptr() as *const f32;
                for i in 0..6 {
                    probs[i] = *pi_ptr.offset(i as isize);
                }
            }
            let a = categorical_sample(&probs);
            let (s_prime, r, d) = env.step(a);
            //println!("{}", a);
            let item = TrainItem {
               state: s,
               action: a as f32,
               reward: r,
               state_prime: s_prime,
               done: if d {0.0} else {1.0},
               probability: probs[a]
            };
            model.put_data(item);
            s = s_prime;
            score += r;
            done = d;            
        }
        scores.push(score as f64);

        if scores.len() >= 100 {
            println!("iter:{}. score: {}", n_epi, scores.iter().sum::<f64>() / scores.len() as f64);
            scores.clear();
        }
            
        model.train_net();
    }
}