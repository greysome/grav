use ggez::nalgebra::{Point2, Vector2};

// All points/vectors are in reference to global xy-plane
// For convenience, 1 unit = 1 metre
#[derive(Debug, Clone, Copy)]
pub struct Body {
    pub mass: f32,
    pub pos: Point2<f32>,
    pub v: Vector2<f32>,
    pub a: Vector2<f32>,
    pub color: [f32; 4]
}

impl Body {
    pub fn accel_towards(&self, other: &Body) -> Vector2<f32> {
        let dx = other.pos.x - self.pos.x;
        let dy = other.pos.y - self.pos.y;

        let r_squared = dx.powi(2) + dy.powi(2);
        if r_squared == 0.0 { return Vector2::new(0.0, 0.0); }

        let g = 6.67e-11_f32;
        let a = g * other.mass / r_squared;

        let theta = dy.atan2(dx);
        Vector2::new(a * theta.cos(), a * theta.sin())
    }
}
