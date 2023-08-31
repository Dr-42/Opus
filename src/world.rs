use crate::organism;
use nalgebra::Vector2;

pub struct World {
    pub organisms: Vec<organism::Organism>,
    pub size: Vector2<usize>,
}
