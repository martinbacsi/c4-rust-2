use minifb::{Key, Window, WindowOptions};

use crate::csb::CSB_Game;

const CP_RAD: f32 = 400.0;
const POD_RAD: f32 = 200.0;
pub struct Visualizer {
    window: Window,
    buffer: Vec<u32>,
    width: usize,
    height: usize,
    ratio: f32
}

impl Visualizer {
    pub fn new(ratio: f32, title: &str) -> Self {
        let width = (16000.0 / ratio) as usize;
        let height = (9000.0 / ratio) as usize;
        let window = Window::new(
            title,
            width,
            height,
            WindowOptions::default(),
        ).unwrap_or_else(|e| panic!("Unable to create window: {}", e));

        let buffer = vec![0; width * height];

        Visualizer {
            window,
            buffer,
            width,
            height,
            ratio
        }
    }

    pub fn draw(&mut self, game: &CSB_Game){
        let cp_rad_s = (CP_RAD / self.ratio) * (CP_RAD / self.ratio);
        let pod_rad_s = (POD_RAD / self.ratio) * (POD_RAD / self.ratio);

        for y in 0..self.height {
            for x in 0..self.width {
                self.buffer[x + y * self.width] =  0x000000;
                for (i, p) in game.map.iter().enumerate() {
                    let cx = p.x / self.ratio;
                    let cy = p.y / self.ratio;
                    let distance = ((x as isize - cx as isize).pow(2) + (y as isize - cy as isize).pow(2)) as f32;
                    
                    if distance < cp_rad_s {
                        if i  == game.pod.cp % game.map.len() {
                            self.buffer[x + y * self.width] = 0x00FF00;
                        } else {
                            self.buffer[x + y * self.width] = 0x0000FF;
                        }
                        break;
                    }
                }

                let cx = game.pod.pos.x / self.ratio;
                let cy = game.pod.pos.y / self.ratio;
                let distance = ((x as isize - cx as isize).pow(2) + (y as isize - cy as isize).pow(2)) as f32;
                
                if distance < pod_rad_s {
                    self.buffer[x+ y * self.width] = 0xFF0000;
                }
            }
        }
    }
    pub fn update(&mut self) {
        self.window
        .update_with_buffer(&self.buffer, self.width, self.height)
        .unwrap();
    }

    pub fn is_open(&self) -> bool {
        self.window.is_open() && !self.window.is_key_down(Key::Escape)
    }
}