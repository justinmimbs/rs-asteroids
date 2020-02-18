use rand::SeedableRng;
use rand_pcg::Pcg32;
use std::iter;

use crate::asteroid::Asteroid;
use crate::blast::Blast;
use crate::geometry::{Polygon, Size};
use crate::motion::{Collide, Movement};
use crate::player::{Controls, Player};

const BOUNDS: Size = Size {
    width: 1200.0,
    height: 900.0,
};

pub struct Level {
    // rng: Pcg32,
    pub player: Player,
    pub asteroids: Vec<Asteroid>,
    pub blasts: Vec<Blast>,
}

impl Level {
    pub fn new(n: u8) -> Self {
        let mut rng = Pcg32::seed_from_u64(1979 * 11 * n as u64);
        let count = 3 + 2 * n as u32;
        Level {
            player: Player::new(BOUNDS.center()),
            asteroids: Asteroid::field(&mut rng, &BOUNDS, count, 100.0),
            blasts: Vec::new(),
            // rng,
        }
    }

    pub fn step(&mut self, dt: f64, controls: Controls) -> () {
        if dt <= 0.0 {
            return ();
        }

        // step

        self.player.step(dt, &BOUNDS, controls);

        for asteroid in self.asteroids.iter_mut() {
            asteroid.step(dt, &BOUNDS);
        }

        self.blasts.extend(self.player.fire_blast());
        for blast in self.blasts.iter_mut() {
            blast.step(dt, &BOUNDS);
        }
        self.blasts.retain(|blast| !blast.is_expired());

        // interact: asteroids * blasts

        let mut asteroids = Vec::new();
        for asteroid in self.asteroids.drain(..) {
            if let Some((i, mut fragments)) = interact_asteroid_blasts(&asteroid, &self.blasts) {
                self.blasts.remove(i);
                asteroids.append(&mut fragments);
            } else {
                asteroids.push(asteroid);
            }
        }
        self.asteroids = asteroids;
    }
}

fn interact_asteroid_blasts(
    asteroid: &Asteroid,
    blasts: &Vec<Blast>,
) -> Option<(usize, Vec<Asteroid>)> {
    for (i, blast) in blasts.iter().enumerate() {
        if let Some(fragments) = interact_asteroid_blast(asteroid, blast) {
            return Some((i, fragments));
        }
    }
    None
}

fn interact_asteroid_blast(asteroid: &Asteroid, blast: &Blast) -> Option<Vec<Asteroid>> {
    let (head, tail) = blast.endpoints();
    if head.distance_squared(asteroid.center()) < asteroid.radius().powi(2) {
        let asteroid_boundary = asteroid.boundary();
        let maybe_impact_point = {
            let intersections =
                Polygon(&asteroid_boundary).intersections(iter::once((&head, &tail)));
            if head < tail {
                intersections.into_iter().min()
            } else {
                intersections.into_iter().max()
            }
        };
        if let Some(impact_point) = maybe_impact_point {
            let impact_speed = blast.velocity().length() * (100.0 / (100.0 + asteroid.mass()));
            let fragments = (Polygon(&asteroid_boundary).split(&head, &tail).iter())
                .map(|fragment_boundary| {
                    let mut fragment = Asteroid::from_polygon(fragment_boundary);
                    let impact_velocity = blast.velocity().normalize().scale(impact_speed);
                    let impact_movement =
                        Movement::from_impulse(fragment.center(), &impact_point, &impact_velocity);
                    let outward_movement = Movement {
                        velocity: (asteroid.center().direction_to(&fragment.center()))
                            .scale(impact_speed),
                        angular_velocity: 0.0,
                    };
                    let movement = outward_movement
                        .interpolate(asteroid.movement(), fragment.mass() / asteroid.mass())
                        .add(&impact_movement);
                    fragment.set_movement(movement);
                    fragment
                })
                .collect();
            Some(fragments)
        } else {
            None
        }
    } else {
        None
    }
}
