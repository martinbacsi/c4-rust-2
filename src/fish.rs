// Assuming you already have the Vector and FishType structs from the previous examples
use crate::{vector::*, entity::* };

#[derive(Debug)]
pub enum FishType {
    JELLY,
    FISH,
    CRAB,
}

trait Entity {
    fn get_pos(&self) -> Vector;
    fn get_speed(&self) -> Vector;
}

#[derive(Debug)]
pub struct Fish {
    pub id: i32,
    pub pos: Vector,
    pub fish_type: FishType,
    pub color: i32,
    pub low_y: i32,
    pub high_y: i32,
    pub speed: Vector,
}

impl Fish {
    pub fn new(x: f64, y: f64, fish_type: FishType, color: i32, id: i32, low_y: i32, high_y: i32) -> Fish {
        Fish {
            id,
            pos: Vector::new(x, y),
            fish_type,
            color,
            low_y,
            high_y,
            speed: Vector::ZERO,
        }
    }

    pub fn get_x(&self) -> f64 {
        self.pos.x
    }

    pub fn get_y(&self) -> f64 {
        self.pos.y
    }
}

impl Entity for Fish {
    fn get_pos(&self) -> Vector {
        self.pos
    }

    fn get_speed(&self) -> Vector {
        self.speed
    }
}