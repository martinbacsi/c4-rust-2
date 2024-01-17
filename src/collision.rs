#[derive(Debug)]
struct Collision {
    t: f64,
    a: Option<Entity>,
    b: Option<Entity>,
}

impl Collision {
    const NONE: Collision = Collision {
        t: -1.0,
        a: None,
        b: None,
    };

    fn new(t: f64) -> Collision {
        Collision { t, a: None, b: None }
    }

    fn with_entity_a(t: f64, a: Entity) -> Collision {
        Collision {
            t,
            a: Some(a),
            b: None,
        }
    }

    fn with_entities(t: f64, a: Entity, b: Entity) -> Collision {
        Collision {
            t,
            a: Some(a),
            b: Some(b),
        }
    }

    fn happened(&self) -> bool {
        self.t >= 0.0
    }
}