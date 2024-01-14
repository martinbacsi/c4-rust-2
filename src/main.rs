use core::panic;
use tch::{nn, Device, Tensor, nn::OptimizerConfig, nn::Adam, nn::VarStore, nn::Optimizer};

mod csb;
mod visu;

use visu::Visualizer;

use crate::csb::CSB_Game;
use rand::prelude::*;

const LEARNING_RATE: f64 = 0.0003;
const GAMMA: f64 = 0.98;
const LAMBDA: f64 = 0.95;
const EPS_CLIP: f32 = 0.1;
const K_EPOCH: usize = 3;

struct TrainItem {
    state: [f32; 11],
    action: f32,
    reward: f32,
    state_prime: [f32; 11],
    done: f32,
    probability: f32,
}

struct PPO {
    fc0: nn::Linear,
    fc1: nn::Linear,
    fc_v: nn::Linear,
    fc_a: nn::Linear,
    optimizer: nn::Optimizer<nn::Adam>,
    vs: VarStore,
    data: Vec<TrainItem>,  
}

impl PPO {
    fn new() -> PPO {
        let mut vs = VarStore::new(tch::Device::Cpu);
        let root = &vs.root();
        let mut ppo = PPO
        {
            fc0: nn::linear(root / "c_fc0", 11, 128, Default::default()),
            fc1: nn::linear(root / "c_fc1", 128, 128, Default::default()),
            fc_a: nn::linear(root / "c_fc_a", 128, 6, Default::default()),
            fc_v: nn::linear(root / "c_fc_v", 128, 1, Default::default()),
            optimizer: nn::Adam::default().build(&vs, LEARNING_RATE).unwrap(),
            vs: vs,
            data: Vec::new()
        };
        //ppo.vs.load("latest.pt").expect("failed to load");
        ppo
    }

    fn run(&self, x: &Tensor) -> (Tensor, Tensor) {       
        let x = x 
        //.flat_view()
        .apply(&self.fc0)
        .relu()
        .apply(&self.fc1)
        .relu();
        let v = x.apply(&self.fc_v);
        let a = x.apply(&self.fc_a).softmax(-1, tch::Kind::Float);
        (a, v)
    } 

    fn put_data(&mut self, transition: TrainItem) {
        self.data.push(transition);
    }

    fn make_batch(&mut self) -> (Tensor, Tensor, Tensor, Tensor, Tensor, Tensor) {
        let data_len = self.data.len() as i64;
        let s = Tensor::zeros(&[data_len, 11], (tch::Kind::Float, tch::Device::Cpu));
        let a = Tensor::zeros(&[data_len, 1], (tch::Kind::Int64, tch::Device::Cpu));
        let r = Tensor::zeros(&[data_len, 1], (tch::Kind::Float, tch::Device::Cpu));
        let s_prime = Tensor::zeros(&[data_len, 11], (tch::Kind::Float, tch::Device::Cpu));
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
                for j in 0..11 as usize {
                    let id = (i * 11 + j) as isize;
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
        let old_log_probs = prob_a.log();
       
        let mut discounted_sum: f32 = 0.0; 
        let mut returns_list = Vec::new();
        
        let old_rewards = &r * &done_mask;
        unsafe {
            let numel = old_rewards.numel();
            let d_ptr = old_rewards.data_ptr() as *const f32;
            for i in 0..numel {
                discounted_sum = ((GAMMA) as f32) * discounted_sum + *d_ptr.offset((numel -  i - 1) as isize);
                returns_list.push(discounted_sum);
            }
        }
        
        returns_list.reverse();
        // Initialize advantage.
        let returns = Tensor::of_slice(&returns_list).view([-1]);
        let returns_norm = (&returns - &returns.mean(tch::Kind::Float)) / (&returns.std(true) + 1e-8);
     
        for _ in 0..K_EPOCH {   
            let (probs, values) = self.run(&s);
            let advantage = &returns_norm - &values.detach();
            let new_probs = probs.gather(1, &a,     false);    
            //pi.print();
            let ratio = (&new_probs.log() - &old_log_probs).exp();
            //ratio.print();
            let surr1 = &ratio * &advantage;        
            let surr2 = ratio.clamp((1.0 - EPS_CLIP) as f64, (1.0 + EPS_CLIP) as f64) * &advantage;      
            
            let epsilon = 1e-8;
            let probs_clamped = new_probs.clamp(epsilon, 1.0 - epsilon);
            let entropy_per_row = -&probs_clamped* probs_clamped.log( );
            let entropy = entropy_per_row.sum1(&[1], true, tch::Kind::Double);


            let actor_loss = -&surr1.min1(&surr2).mean(tch::Kind::Float);// + entropy.mean(tch::Kind::Float) * 0.01;
            
            let critic_loss = (&returns_norm - &values).pow(2).mean(tch::Kind::Float);
          
            let loss = actor_loss + critic_loss;
            
            self.optimizer.zero_grad();

            loss.backward();
            self.optimizer.step();
        
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

    let mut laps: Vec<f64> = Vec::new();
    for n_epi in 0..100000 {
        let mut score = 0.0;
        let mut env = CSB_Game::new();
        let mut done = false;
        let mut s = env.encode();
       
        let mut turn = 0;
        
        while !done {
            turn = turn + 1;
            if n_epi % 100 == 0 {
                visu.draw(&env);
                visu.update();
            }
            let (pi, v) = model.run(&Tensor::of_slice(&s).view([1, 11]));

            //pi.print();

            unsafe {
                let pi_ptr = pi.data_ptr() as *const f32;
                for i in 0..6 {
                    probs[i] = *pi_ptr.offset(i as isize);
                }
            }
            //pi.print();
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
        if env.pod.cp > 0 {
            laps.push(turn as f64 / (env.pod.cp as f64));
        }
        scores.push( env.pod.cp  as f64 / (env.map.len() as f64 * 3.0));

        model.train_net();
        if scores.len() >= 100 {
            println!("iter:{}. score: {}, laptime: {}", n_epi, scores.iter().sum::<f64>() / scores.len() as f64, laps.iter().sum::<f64>() / laps.len() as f64);
            scores.clear();
            laps.clear();
            
            model.vs.save("latest.pt");
        }        
    }
}