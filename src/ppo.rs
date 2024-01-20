use tch::{nn, Device, Tensor, nn::OptimizerConfig, nn::Adam, nn::VarStore, nn::Optimizer,};
use crate::{game::*, vector::Vector};

use crate::csb::CSB_Game;
use rand::prelude::*;
use crate::xorshift::*;

const LEARNING_RATE: f64 = 0.0003;
const GAMMA: f64 = 0.98;
const EPS_CLIP: f32 = 0.1;
const K_EPOCH: usize = 3;

const ACTION_SIZE: usize = 16;

struct TrainItem {
    state: [f32; STATE_SIZE],
    action: f32,
    reward: f32,
    state_prime: [f32;STATE_SIZE],
    done: f32,
    probability: f32,
}

struct Actor {
    fc0: nn::Linear,
    fc1: nn::Linear,
    fc2: nn::Linear,
    fc3: nn::Linear,
    fc4: nn::Linear,
    fc_pi: nn::Linear,
}

impl Actor {
    fn run(&self, x: &Tensor) -> Tensor {       
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
}
struct Critic {
    fc0: nn::Linear,
    fc1: nn::Linear,
    fc2: nn::Linear,
    fc3: nn::Linear,
    fc4: nn::Linear,
    fc_v: nn::Linear,
}

impl Critic {
    fn run(&self, x: &Tensor) -> Tensor {       
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
       let v = x.apply(&self.fc_v);
       v
   } 
}
struct PPO {
    actor: Actor,
    cricic: Critic,
    optimizer: nn::Optimizer<nn::Adam>,
    data: Vec<TrainItem>,
    vs: VarStore
}

impl PPO {
    fn new() -> PPO {
        let mut vs = VarStore::new(tch::Device::Cpu);
        let root = &vs.root();
        let actor = Actor 
        {
            fc0: nn::linear(root / "a_fc0", STATE_SIZE as i64, 128, Default::default()),
            fc1: nn::linear(root / "a_fc1", 128, 128, Default::default()),
            fc2: nn::linear(root / "a_fc2", 128,128, Default::default()),
            fc3: nn::linear(root / "a_fc3", 128, 128, Default::default()),
            fc4: nn::linear(root / "a_fc4", 128, 128, Default::default()),
            fc_pi: nn::linear(root / "a_fc_pi", 128, ACTION_SIZE as i64, Default::default()),
        };

        let cricic = Critic 
        {
            fc0: nn::linear(root / "c_fc0", STATE_SIZE as i64, 128, Default::default()),
            fc1: nn::linear(root / "c_fc1", 128, 128, Default::default()),
            fc2: nn::linear(root / "c_fc2", 128, 128, Default::default()),
            fc3: nn::linear(root / "c_fc3", 128, 128, Default::default()),
            fc4: nn::linear(root / "c_fc4", 128, 128, Default::default()),
            fc_v: nn::linear(root / "c_fc_v", 128, 1, Default::default()),
        };

        let optimizer = nn::Adam::default().build(&vs, LEARNING_RATE).unwrap();
     
        //vs.load("model_fish.pt").expect("failed to load");
        
        PPO { actor, cricic, optimizer, data: Vec::new(), vs }
    }

    fn pi(&self, x: &Tensor) -> Tensor {     
        
        self.actor.run(x)
    } 

    fn v(&self, x: &Tensor) -> Tensor{       
        self.cricic.run(x)
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

    fn train_net(&mut self) {
        let (s, a, r, s_prime, done_mask, prob_a) = self.make_batch();
        
        for _ in 0..K_EPOCH {         
            let td_target = &r + &self.v(&s_prime).detach() * done_mask.detach();
           
            let delta = (&td_target - &self.v(&s)).detach();
           
            //(&r + &self.v(&s_prime)).print();
            let mut advantage: f32 = 0.0; 
            let mut advantage_list = Vec::new();

            unsafe {
                let numel = delta.numel();
                let d_ptr = delta.data_ptr() as *const f32;
                for i in 0..numel {
                    advantage = ((GAMMA) as f32) * advantage + *d_ptr.offset((numel -  i - 1) as isize);
                    advantage_list.push(advantage);
                }
            }
            advantage_list.reverse();
            // Initialize advantage.
            let advantage_tensor = Tensor::of_slice(&advantage_list).view([-1]);
            
           
            let pi = &self.pi(&s);
            let pi_a = pi.gather(1, &a, false);    
            
            let ratio = (&pi_a.log() - &prob_a.log()).exp();

            let surr1 = &ratio * &advantage_tensor;        
            let surr2 = ratio.clamp((1.0 - EPS_CLIP) as f64, (1.0 + EPS_CLIP) as f64) * &advantage_tensor;
            let l1 =  (&self.v(&s) - &td_target.detach()).pow(2.0).mean(tch::Kind::Float);
            let loss = (-surr1.min1(&surr2)).mean(tch::Kind::Float) + l1;
            self.optimizer.zero_grad();
            loss.backward();
            self.optimizer.step();
            self.vs.save("model_fish.pt");
        }
    }
}

fn categorical_sample(probs: &[f32; ACTION_SIZE]) -> usize {
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

pub fn run_ppo() {
    let mut model = PPO::new();
    let mut probs: [f32; ACTION_SIZE] = [0.0; ACTION_SIZE];

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
            let pi = model.pi(&Tensor::of_slice(&s).view([1, 64]));
            //eprintln!("pi ran");
            unsafe {
                let pi_ptr = pi.data_ptr() as *const f32;
                for i in 0..ACTION_SIZE {
                    probs[i] = *pi_ptr.offset(i as isize);
                }
            }
            let a = categorical_sample(&probs);
            let light = a % 2 == 0;
            let angle_index = a / 2;
            let angle = 2.0 * std::f64::consts::PI * angle_index  as f64 / ((ACTION_SIZE as f64) / 2.0);

            let dir = Vector::new(angle.cos() * 10000.0, angle.sin() * 10000.0);
            //eprintln!("{} {}", dir.x, dir.y );
            let (s_prime, r, d) = env.step(env.players[0].drones[0].pos.add(dir), light);
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
        eprintln!("score: {}, turns: {}", env.score(0), turn);      
            
        model.train_net();
    }
}