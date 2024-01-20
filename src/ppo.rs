use tch::{nn, Device, Tensor, nn::OptimizerConfig, nn::Adam, nn::VarStore, nn::Optimizer,};
use crate::{game::*, vector::Vector};

use crate::csb::CSB_Game;
use rand::prelude::*;
use crate::xorshift::*;

const LEARNING_RATE: f64 = 0.0003;
const GAMMA: f64 = 0.99;
const LAMBDA: f64 = 0.95;
const EPS_CLIP: f32 = 0.1;
const K_EPOCH: usize = 3;


const FISH_VECTOR_LENGTH: i64 = 3;
struct TrainItem {
    state: [f32; STATE_SIZE],
    action: f32,
    reward: f32,
    state_prime: [f32;STATE_SIZE],
    done: f32,
    probability: f32,
}


pub struct PPO {
    vs: VarStore,
    optimizer: nn::Optimizer<nn::Adam>,
    data: Vec<TrainItem>,
    fc0: nn::Linear,
    fc1: nn::Linear,
    fc2: nn::Linear,
    fc3: nn::Linear,
    fc_pi_0: nn::Linear,
    fc_pi_1: nn::Linear,
    fc_v_0: nn::Linear,
    fc_v_1: nn::Linear,
}

impl PPO {
    pub fn new() -> PPO {
        let vs = VarStore::new(tch::Device::Cpu);
        let root = &vs.root();
        let optimizer = nn::Adam::default().build(&vs, LEARNING_RATE).unwrap();

        let fc0 = nn::linear(root / "c_fc0", INPUT_PER_FISH as i64, 32, Default::default());
        let fc1 = nn::linear(root / "c_fc1", 32, FISH_VECTOR_LENGTH, Default::default());

        let fc2 = nn::linear(root / "c_fc2", FISH_VECTOR_LENGTH * 12 + INPUT_PER_DRONE as i64 * Game::DRONES_PER_PLAYER as i64, 32, Default::default());
        let fc3 = nn::linear(root / "c_fc3", 32, 32, Default::default());
        let fc_pi_0 = nn::linear(root / "c_fc_pi_0", 32, 32, Default::default());
        let fc_pi_1 = nn::linear(root / "c_fc_pi_1", 32, ACTION_SIZE as i64, Default::default());
        let fc_v_0 = nn::linear(root / "c_fc_v_0", 32, 32, Default::default());
        let fc_v_1 = nn::linear(root / "c_fc_v_1", 32, 1, Default::default());

        let mut ppo = PPO {
            vs,
            optimizer,
            data: Vec::new(),
            fc0,
            fc1,
            fc2,
            fc3,
            fc_pi_0,
            fc_pi_1,
            fc_v_0,
            fc_v_1,
        };

        ppo.vs.load("model_fish.pt").expect("failed to load");
        ppo
    }

    pub fn base_net(&self, x: &Tensor) -> Tensor {
        let splt = x.split(INPUT_PER_FISH as i64 * 12, -1);
        let fishes = splt.get(0).unwrap();
        let drone = splt.get(1).unwrap();
        let fishes_batch = &fishes.reshape(&[-1, INPUT_PER_FISH as i64]);
        let fishes_processed = fishes_batch.apply(&self.fc0)
        .relu()
        .apply(&self.fc1)
        .relu();

        let fishes_backshaped = fishes_processed.view([-1, INPUT_PER_FISH as i64 * 12]);
        let fishesprocess_and_drones =  Tensor::cat(&[&fishes_backshaped, &drone], 1);

        fishesprocess_and_drones
            .apply(&self.fc2)
            .relu()
            .apply(&self.fc3)
            .relu()
    }

    pub fn pi(&self, x: &Tensor) -> Tensor {    
        self.base_net(x)
        .apply(&self.fc_pi_0)
        .relu()
        .apply(&self.fc_pi_1).softmax(-1, tch::Kind::Float)
    } 

    pub fn v(&self, x: &Tensor) -> Tensor{     
        self.base_net(x)
        .apply(&self.fc_v_0)
        .relu()
        .apply(&self.fc_v_1)
    }

    fn put_data(&mut self, transition: TrainItem) {
        self.data.push(transition);
    }

    fn make_batch(&mut self) -> (Tensor, Tensor, Tensor, Tensor, Tensor, Tensor) {
        let data_len = self.data.len() as i64;
        let s = Tensor::zeros(&[data_len, STATE_SIZE as i64], (tch::Kind::Float, tch::Device::Cpu));
        let a = Tensor::zeros(&[data_len, 1], (tch::Kind::Int64, tch::Device::Cpu));
        let r = Tensor::zeros(&[data_len, 1], (tch::Kind::Float, tch::Device::Cpu));
        let s_prime = Tensor::zeros(&[data_len, STATE_SIZE as i64], (tch::Kind::Float, tch::Device::Cpu));
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
                for j in 0..STATE_SIZE as usize {
                    let id = (i * STATE_SIZE + j) as isize;
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

    fn train_net(&mut self, save: bool) {
        let (s, a, r, s_prime, done_mask, prob_a) = self.make_batch();
        
        for _ in 0..K_EPOCH {         
            let td_target = &r + &self.v(&s_prime).detach() * done_mask.detach() * GAMMA;
           
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
            
           
            let pi = &self.pi(&s);
            let pi_a = pi.gather(1, &a, false);    
            
            let ratio = &pi_a / (&prob_a + 1e-6);

            let surr1 = &ratio * &advantage_tensor;        
            let surr2 = ratio.clamp(    (1.0 - EPS_CLIP) as f64, (1.0 + EPS_CLIP) as f64) * &advantage_tensor;
            
            let l1 =  (&self.v(&s) - &td_target.detach()).abs().mean(tch::Kind::Float);
            let loss = (-surr1.min1(&surr2)).mean(tch::Kind::Float) + l1;
            self.optimizer.zero_grad();
            loss.backward();
            //loss.print();
            self.optimizer.step();
            if save {
                self.vs.save("model_fish.pt");
            }
        }
    }
}

pub fn categorical_sample(probs: &Tensor) -> usize {
    unsafe {
        let pi_ptr = probs.data_ptr() as *const f32;
        let mut rng = rand::thread_rng();
        let rand_value: f32 = rng.gen_range(0.0..1.0); 
        let mut cumulative_prob = 0.0;
        for i in 0..ACTION_SIZE {
            cumulative_prob += *pi_ptr.offset(i as isize);
            if rand_value <= cumulative_prob {
                return i;
            }
        }
    }
    panic!();
}

pub fn run_ppo() {
    let mut model = PPO::new();

    let mut rng = xorshift::new(69);
    for n_epi in 0..100000 {
        let mut score = 0.0;
        let mut env = Game::new(rng.next());
        let mut done = false;
        let mut s = env.encode();
       
        let mut turn = 0;
        
        while !done {
            turn = turn + 1;
            //eprintln!("run pi");
            //let tr = Tensor::of_slice(&s).transpose(1, STATE_SIZE as i64);
             //eprintln!("reshaped");
            //Tensor::of_slice(&s).print();
            let pi = model.pi(&Tensor::of_slice(&s).view([1, STATE_SIZE as i64])).view([ACTION_SIZE as i64]);
            //eprintln!("pi ran");
            let a = categorical_sample(&pi);

            let (dir, light) = decode_action(a as i64);
            //eprintln!("{} {}", dir.x, dir.y );
            let (s_prime, r, d) = env.step(dir, light);
            //println!("{}", a);
            let item = TrainItem {
               state: s,
               action: a as f32,
               reward: r,
               state_prime: s_prime,
               done: if d {0.0} else {1.0},
               probability: pi.double_value(&[a as i64]) as f32
            };
            model.put_data(item);
            s = s_prime;
            score += r;
            done = d; 
           
        }

        eprintln!("score: {}, value:{}, turns: {}", env.score(0), model.v(&Tensor::of_slice(&s).view([1, STATE_SIZE as i64])).double_value(&[]), turn);      
            
        model.train_net(n_epi % 1000 == 0);
    }
}