use rand::{self, RngCore};
use rand::seq::SliceRandom;
use rand::Rng;
use std::ops::Sub;

const LEFT_ACC: usize = 0;
const RIGHT_MOVE: usize = 1;
const STRAIGHT_ACC: usize = 2;
const STRAIGHT_MOVE: usize = 3;
const RIGHT_ACC: usize = 4;
const LEFT_MOVE: usize = 5;

const MAX_SPEED: f32 = 650.0;

#[derive(Debug, Copy, Clone)]
pub struct V2D {
    pub x: f32,
    pub y: f32,
}

impl Sub for V2D {
    type Output = V2D;

    fn sub(self, rhs: V2D) -> V2D {
        V2D {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
        }
    }
}

impl V2D {
    fn new(x: f32, y: f32) -> Self {
        V2D { x, y }
    }

    fn rotate(&mut self, angle: f32) {
        let r = angle.to_radians();
        let cs = r.cos();
        let sn = r.sin();

        let px = self.x * cs - self.y * sn;
        self.y = self.x * sn + self.y * cs;
        self.x = px;
    }

    fn dist(&self, v: V2D) -> f32 {
        let d = *self - v;
        (d.x * d.x + d.y * d.y).sqrt()
    } 
}

pub struct Pod {
    pub pos: V2D,
    angle: f32,
    v: V2D,
    time: i32,
    pub cp: usize,
}

impl Pod {
    fn new() -> Self {
        let mut pod = Pod {
            pos: V2D::new( rand::thread_rng().gen_range(0.0..16000.0), rand::thread_rng().gen_range(0.0..9000.0)),
            angle: rand::thread_rng().gen_range(0.0..359.0),
            v: V2D::new(rand::thread_rng().gen_range(0.0..MAX_SPEED as f32), 0.0),
            time: 200,
            cp: 0,
        };
        pod.v.rotate(rand::thread_rng().gen_range(0.0..359.0));
        pod.v.x = pod.v.x.trunc();
        pod.v.y = pod.v.y.trunc();
        pod
    }
    
    fn set_input(&mut self, x: f32, y: f32, vx: f32, vy: f32, angle: f32, next_check_point_id: usize) {
        self.pos = V2D::new(x, y);
        self.v = V2D::new(vx, vy);
        self.angle = angle;
        self.cp = next_check_point_id;
    }
    
    fn apply(&mut self, a: usize) {
        if a == LEFT_ACC || a == LEFT_MOVE {
            self.angle = self.angle - 18.0;
        }
        
        if a == RIGHT_ACC || a == RIGHT_MOVE {
            self.angle = self.angle + 18.0;
        }

        if self.angle < 0.0 {
            self.angle += 360.0;
        }

        if self.angle > 360.0 {
            self.angle -= 360.0;
        }
        
        if a == LEFT_ACC || a == RIGHT_ACC || a == STRAIGHT_ACC {
            let ra = self.angle.to_radians();
            let thrust = 100.0;
            self.v.x = self.v.x + ra.cos() * thrust;
            self.v.y = self.v.y + ra.sin() * thrust;     
        }
    }
    
    fn end(&mut self) {
        self.v.x = self.v.x * 0.85;
        self.v.y = self.v.y * 0.85;
        self.pos.x = self.pos.x.round();
    }

    fn move_pod(&mut self) {
        let t = 1.0;
        self.pos.x = (self.pos.x + self.v.x * t).round();
        self.pos.y = (self.pos.y + self.v.y * t).round();
    }

    fn encode(&self, map: &Vec<V2D>) -> [f32; 11] {
        let next_cp = map[self.cp % map.len()];
        let mut next_next_id = if self.cp + 1 == map.len() * 3 { self.cp } else {self.cp + 1};
        next_next_id = next_next_id % map.len() ;
        let next_next_cp = map[next_next_id];

        let inputs: [f32; 11] = [
            self.angle.to_radians().sin(),
            self.angle.to_radians().cos(),
            self.v.x as f32 / MAX_SPEED,
            self.v.y as f32 / MAX_SPEED,
            self.pos.x as f32 / 16000.0,
            self.pos.y as f32 / 16000.0,
            next_cp.x as f32 / 16000.0,
            next_cp.y as f32 / 16000.0,
            next_next_cp.x as f32 / 16000.0,
            next_next_cp.y as f32 / 16000.0,
            self.cp as f32 / 100.
        ];

        inputs
    }
}

fn collision2(p1: V2D, p2: V2D, v1: V2D, v2: V2D, is_cp: bool) -> f32 {
    if v1.x == v2.x && v1.y == v2.y {
        return 1.0;
    }

    //let sr2 = if is_cp { 357604.0 } else { 640000.0 };
    let sr2 = if is_cp { 1500.0 * 1500.0 } else { 640000.0 };

    let dp = p1 - p2;
    let dv = v1 - v2;
    let a = dv.x.powi(2) + dv.y.powi(2);

    if a < 0.00001 {
        return 1.0;
    }

    let b = -2.0 * (dp.x * dv.x + dp.y * dv.y);
    let delta = b.powi(2) - 4.0 * a * (dp.x.powi(2) + dp.y.powi(2) - sr2);

    if delta < 0.0 {
        return 1.0;
    }

    let t = (b - delta.sqrt()) * (1.0 / (2.0 * a));

    if t <= 0.0 || t > 1.0 {
        return 1.0;
    }

    t
}

pub struct CSB_Game {
    action_space: usize,
    state_size: usize,
    pub pod: Pod,
    pub map: Vec<V2D>,
}

impl CSB_Game {
    const ACTION_SPACE: usize = 6;
    const STATE_SIZE: usize = 11;

    pub fn encode(&self) ->  [f32; 11]  {
       self.pod.encode(&self.map)
    }

    pub fn new() -> Self {
         let maps: Vec<Vec<(i32, i32)>> = vec![
            vec![(12460, 1350), (10540, 5980), (3580, 5180), (13580, 7600)],
            vec![(3600, 5280), (13840, 5080), (10680, 2280), (8700, 7460), (7200, 2160)],
            vec![(4560, 2180), (7350, 4940), (3320, 7230), (14580, 7700), (10560, 5060), (13100, 2320),],
            vec![(5010, 5260), (11480, 6080), (9100, 1840)],
            vec![(14660, 1410), (3450, 7220), (9420, 7240), (5970, 4240)],
            vec![(3640, 4420), (8000, 7900), (13300, 5540), (9560, 1400)],
            vec![(4100, 7420), (13500, 2340), (12940, 7220), (5640, 2580)],
            vec![(14520, 7780), (6320, 4290),(7800, 860),(7660, 5970),(3140, 7540),(9520, 4380),],
            vec![(10040, 5970), (13920, 1940), (8020, 3260), (2670, 7020)],
            vec![(7500, 6940), (6000, 5360), (11300, 2820)],
            vec![(4060, 4660),(13040, 1900),(6560, 7840),(7480, 1360),(12700, 7100),],
            vec![(3020, 5190), (6280, 7760),(14100, 7760),(13880, 1220),(10240, 4920),(6100, 2200),],
            vec![ (10323, 3366), (11203, 5425),(7259, 6656),(5425, 2838),],
        ];
        let chosen_map = maps.choose(&mut rand::thread_rng()).unwrap();
        let mut chosen_map_v2d: Vec<V2D> = chosen_map.iter().map(|&(x, y)| V2D::new(x as f32, y as f32) ).collect();

        if rand::thread_rng().next_u32() % 2 == 0 {
            chosen_map_v2d.reverse();
        }

        let game = CSB_Game {
            action_space: Self::ACTION_SPACE,
            state_size: Self::STATE_SIZE,
            pod: Pod::new(),
            map: chosen_map_v2d,
        };
        game
    }

    pub fn step(&mut self, a: usize) -> ([f32; 11], f32, bool) {
        let last_dist =  self.map[self.pod.cp % self.map.len()].dist(self.pod.pos);
        self.pod.apply(a);

        let cp = self.map[self.pod.cp % self.map.len()];
        let mut reward = 0.0;

        if collision2(self.pod.pos, cp, self.pod.v, V2D::new(0.0, 0.0), true) < 1.0 {
            self.pod.cp += 1;
            self.pod.time = 200;
            reward = 100.0;
        }
        
        self.pod.move_pod();
        self.pod.end();
        let new_dist  =  self.map[self.pod.cp % self.map.len()].dist(self.pod.pos);
        if reward < 100.0 {
            reward += (last_dist - new_dist) / 1600.0;
            //println!("{}",reward);
        }
        self.pod.time -= 1;
        let done = self.pod.time < 0 || self.pod.cp == self.map.len() * 3;
        let next_state = self.pod.encode(&self.map);

        (next_state, reward, done)
    }
}