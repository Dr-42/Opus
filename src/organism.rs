use nalgebra::Vector2;
use rand::Rng;

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct BodySquare {
    pub position: Vector2<f64>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Body {
    squares: Vec<BodySquare>,
}

impl Body {
    pub fn new() -> Self {
        Self {
            squares: Vec::new(),
        }
    }

    pub fn add_square(&mut self, square: BodySquare) {
        self.squares.push(square);
    }

    pub fn check_blueprint_validity(&self, proposed_squares: &[BodySquare]) -> bool {
        for square in proposed_squares {
            if !self.is_adjacent(square) {
                return false;
            }
        }
        true
    }

    fn is_adjacent(&self, square: &BodySquare) -> bool {
        self.squares.iter().any(|existing_square| {
            let diff = square.position - existing_square.position;
            diff.magnitude() < 1.5
        })
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum AttributeType {
    MaxEnergy(isize),
    MaxAge(isize),
    MaxSize(isize),
    ReproductionRate(f32),
    MutationRate(f32),
    PubertyAge(isize),
    BodyStates(Vec<Body>),
    Metabolism(f32),
}

#[derive(Debug, PartialEq, Clone)]
pub struct Attribute {
    pub max_energy: isize,
    pub max_age: isize,
    pub max_size: isize,
    pub reproduction_rate: f32,
    pub mutation_rate: f32,
    pub puberty_age: isize,
    pub body_states: Vec<Body>,
    pub metabolism: f32,
}

impl Attribute {
    fn default_attributes() -> Attribute {
        Attribute {
            max_energy: 1000,
            max_age: 1000,
            max_size: 1000,
            reproduction_rate: 0.1,
            mutation_rate: 0.1,
            puberty_age: 100,
            body_states: Vec::new(),
            metabolism: 0.1,
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct Gene {
    pub id: isize,
    pub name: String,
    pub value: isize,
    pub attribute_type: AttributeType,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Genome {
    pub genes: Vec<Gene>,
}

pub struct Organism {
    pub id: isize,
    pub genome: Genome,
    pub energy: isize,
    pub age: isize,
    pub location: Vector2<isize>,
    pub body_squares: Body,
    pub current_body_state: isize,
    pub attributes: Attribute,
}

pub enum OrganismState {
    Alive,
    Dead,
}

impl Organism {
    pub fn apply_gene_effects(&mut self) {
        for gene in &self.genome.genes {
            match &gene.attribute_type {
                AttributeType::MaxEnergy(value) => self.attributes.max_energy += *value,
                AttributeType::MaxAge(value) => self.attributes.max_age += *value,
                AttributeType::MaxSize(value) => self.attributes.max_size += *value,
                AttributeType::ReproductionRate(value) => {
                    self.attributes.reproduction_rate += *value
                }
                AttributeType::MutationRate(value) => self.attributes.mutation_rate += *value,
                AttributeType::PubertyAge(value) => self.attributes.puberty_age += *value,
                AttributeType::BodyStates(value) => {
                    self.attributes.body_states.extend(value.iter().cloned())
                }
                AttributeType::Metabolism(value) => self.attributes.metabolism += *value,
            }
        }
    }

    fn calculate_movement(current_body: &Body, new_body: &Body) -> Vector2<isize> {
        let mut movement = Vector2::zeros();

        // Calculate the average direction of change in controlled squares
        for (current_sq, new_sq) in current_body.squares.iter().zip(new_body.squares.iter()) {
            let delta = new_sq.position - current_sq.position;
            movement += Vector2::new(delta.x as isize, delta.y as isize);
        }

        // Adjust movement based on the difference between controlled squares
        let scaling_factor = -2; // Adjust this to control movement strength
        movement *= scaling_factor;

        movement
    }

    fn mutate(&mut self) {
        let mut rng = rand::thread_rng();
        let mut new_body_states: Vec<Body> = Vec::new();
        for body in &self.attributes.body_states {
            let mut new_body = Body::new();
            for square in &body.squares {
                let mut new_square = square.clone();
                let x = new_square.position.x as f64;
                let y = new_square.position.y as f64;
                let x = x + rng.gen_range(-1.0..1.0);
                let y = y + rng.gen_range(-1.0..1.0);
                new_square.position = Vector2::new(x, y);
                new_body.add_square(new_square);
            }
            new_body_states.push(new_body);
        }
        self.attributes.body_states = new_body_states;
    }

    pub fn new(id: isize, genome: Genome) -> Self {
        let attributes = Attribute::default_attributes();
        let body = attributes.body_states[0].clone();
        let mut organism = Self {
            id,
            genome,
            energy: 1000,
            age: 0,
            location: Vector2::new(0, 0),
            body_squares: body,
            current_body_state: 0,
            attributes,
        };
        organism.apply_gene_effects();
        organism
    }

    pub fn reproduce(&self) -> Organism {
        let location_offset_x = rand::thread_rng().gen_range(-1..1);
        let location_offset_y = rand::thread_rng().gen_range(-1..1);
        let location_offset = Vector2::new(location_offset_x, location_offset_y);
        let mut offspring = Self {
            id: self.id,
            genome: self.genome.clone(),
            energy: self.energy / 2,
            age: 0,
            location: self.location + location_offset,
            body_squares: self.body_squares.clone(),
            current_body_state: 0,
            attributes: self.attributes.clone(),
        };

        offspring.mutate();
        offspring
    }

    pub fn next_frame(&mut self) -> (OrganismState, Option<Organism>) {
        let prev_body = &self.body_squares;
        let next_body = match &self
            .attributes
            .body_states
            .get(self.current_body_state as usize)
        {
            Some(body) => *body,
            None => &self.attributes.body_states[0],
        };
        let ds = Self::calculate_movement(prev_body, next_body);
        self.location += Vector2::new(ds.x as isize, ds.y as isize);
        if self.current_body_state < self.attributes.body_states.len() as isize - 1 {
            self.current_body_state += 1;
        } else {
            self.current_body_state = 0;
        }
        self.energy -=
            (self.attributes.metabolism * self.body_squares.squares.len() as f32) as isize;

        if self.energy <= 0 {
            return (OrganismState::Dead, None);
        }

        self.age += 1;
        if self.age >= self.attributes.max_age {
            return (OrganismState::Dead, None);
        }
        let will_mutate = rand::thread_rng().gen_range(0.0..1.0) < self.attributes.mutation_rate;
        if will_mutate {
            self.mutate();
        }
        let will_reproduce =
            rand::thread_rng().gen_range(0.0..1.0) < self.attributes.reproduction_rate;

        let mut abort = false;
        let offspring = self.reproduce();
        if will_reproduce {
            self.energy -= offspring.energy;

            for body in &offspring.attributes.body_states {
                if !self.body_squares.check_blueprint_validity(&body.squares) {
                    abort = true;
                    break;
                }
            }
        }

        if abort {
            (OrganismState::Alive, None)
        } else if will_reproduce {
            (OrganismState::Alive, Some(offspring))
        } else {
            (OrganismState::Alive, None)
        }
    }
}
