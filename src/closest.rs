use std::ops::Div;

// Assuming you already have the Entity and Vector structs from the previous examples

#[derive(Debug)]
struct Closest<T: Entity> {
    list: Vec<T>,
    distance: f64,
}

impl<T: Entity> Closest<T> {
    fn new(list: Vec<T>, distance: f64) -> Closest<T> {
        Closest { list, distance }
    }

    fn get(&self) -> Option<&T> {
        if self.has_one() {
            Some(&self.list[0])
        } else {
            None
        }
    }

    fn has_one(&self) -> bool {
        self.list.len() == 1
    }

    fn get_pos(&self) -> Option<Vector> {
        if !self.has_one() {
            None
        } else {
            Some(self.list[0].get_pos())
        }
    }

    fn get_mean_pos(&self) -> Option<Vector> {
        if self.has_one() {
            self.get_pos()
        } else {
            let (sum_x, sum_y) = self
                .list
                .iter()
                .map(|entity| entity.get_pos())
                .fold((0.0, 0.0), |acc, pos| (acc.0 + pos.x, acc.1 + pos.y));

            let mean_x = sum_x / self.list.len() as f64;
            let mean_y = sum_y / self.list.len() as f64;

            Some(Vector::new(mean_x, mean_y))
        }
    }
}