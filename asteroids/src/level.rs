use rand::SeedableRng;
use rand_pcg::Pcg32;
use std::iter;

use crate::asteroid::Asteroid;
use crate::blast::Blast;
use crate::geometry::{Polygon, Size};
use crate::iter::EdgesCycleIterator;
use crate::motion::{Collide, Movement};
use crate::particle::{Dispersion, Particle};
use crate::player::{Controls, Player};

const BOUNDS: Size = Size {
    width: 1200.0,
    height: 900.0,
};

pub struct Level {
    rng: Pcg32,
    pub player: Player,
    pub asteroids: Vec<Asteroid>,
    pub blasts: Vec<Blast>,
    pub particles: Vec<Particle>,
}

impl Level {
    pub fn new(n: u8) -> Self {
        let mut rng = Pcg32::seed_from_u64(1979 * 11 * n as u64);
        let count = 3 + 2 * n as u32;
        Level {
            player: Player::new(BOUNDS.center()),
            asteroids: Asteroid::field(&mut rng, &BOUNDS, count, 100.0),
            blasts: Vec::new(),
            particles: Vec::new(),
            rng,
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

        for particle in self.particles.iter_mut() {
            particle.step(dt, &BOUNDS);
        }
        self.particles.retain(|particle| !particle.is_expired());

        // interact: asteroids * blasts

        let mut asteroids = Vec::new();
        for asteroid in self.asteroids.drain(..) {
            if let Some((i, mut impacted)) =
                interact_asteroid_blasts(&mut self.rng, &asteroid, &self.blasts)
            {
                self.blasts.remove(i);
                asteroids.append(&mut impacted.fragments);
                self.particles.append(&mut impacted.particles);
            } else {
                asteroids.push(asteroid);
            }
        }
        self.asteroids = asteroids;
    }
}

fn interact_asteroid_blasts(
    rng: &mut Pcg32,
    asteroid: &Asteroid,
    blasts: &Vec<Blast>,
) -> Option<(usize, ImpactedAsteroid)> {
    for (i, blast) in blasts.iter().enumerate() {
        if let Some(impacted) = interact_asteroid_blast(rng, asteroid, blast) {
            return Some((i, impacted));
        }
    }
    None
}

struct ImpactedAsteroid {
    fragments: Vec<Asteroid>,
    particles: Vec<Particle>,
}

fn interact_asteroid_blast(
    rng: &mut Pcg32,
    asteroid: &Asteroid,
    blast: &Blast,
) -> Option<ImpactedAsteroid> {
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
            let mut fragments = Vec::new();
            let mut particles = Dispersion::new(
                impact_point.clone(),
                asteroid.movement().velocity.clone(),
                100.0,
                50.0,
            )
            .burst(rng, (asteroid.radius() / 4.0).ceil() as u32);

            for fragment_boundary in Polygon(&asteroid_boundary).split(&head, &tail).iter() {
                let mut fragment = Asteroid::from_polygon(fragment_boundary);
                fragment.set_movement({
                    let impact_velocity = blast.velocity().normalize().scale(impact_speed);
                    let impact_movement =
                        Movement::from_impulse(fragment.center(), &impact_point, &impact_velocity);
                    let outward_movement = Movement {
                        velocity: (asteroid.center().direction_to(&fragment.center()))
                            .scale(impact_speed),
                        angular_velocity: 0.0,
                    };
                    outward_movement
                        .interpolate(asteroid.movement(), fragment.mass() / asteroid.mass())
                        .add(&impact_movement)
                });

                if fragment.radius() < 18.0 {
                    let mut fragment_particles = Dispersion::new(
                        fragment.center().clone(),
                        fragment.movement().velocity.clone(),
                        impact_speed,
                        impact_speed,
                    )
                    .explode(
                        rng,
                        (fragment.boundary().iter())
                            .map(|point| point.sub(fragment.center()))
                            .edges_cycle(),
                    );
                    particles.append(&mut fragment_particles);
                } else {
                    fragments.push(fragment);
                }
            }

            Some(ImpactedAsteroid {
                fragments,
                particles,
            })
        } else {
            None
        }
    } else {
        None
    }
}
