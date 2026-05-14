use rand::Rng;
use rand_distr::{Distribution, Normal};
use std::f32::consts::E;
use pyo3::prelude::*;

#[pyclass]
#[derive(Clone, Copy)] // Added Copy to make swapping easier
enum Tire {
    Soft(f32),
    Medium(f32),
    Hard(f32),
}

#[derive(FromPyObject, Clone, Copy)] // Added Clone/Copy for easier track passing
struct Track {
    laps: f32,
    length: f32,
    overtake: f32,
    pit: f32,
    ideallap: f32,
}

#[derive(FromPyObject, Clone)] // Added Clone to support swapping
struct Driver {
    number: u8,
    lap: f32,
    tire: Tire,
    fuel: f32,
    totaltime: f32,
    optlap: f32,
    isstuck: bool,
    strategy: Vec<Strat>,
}

#[derive(FromPyObject, Clone)]
struct Strat {
    tire: Tire,
    target: f32,
    sd: f32,
}

impl Driver {
    fn tiremod(&mut self, defaultmod: f32) -> f32 {
        let mut rng = rand::thread_rng();
        let normal = Normal::new(0.0, 0.3).unwrap();
        let random: f32 = normal.sample(&mut rng);

        let mut modifier = defaultmod;
        if self.isstuck { modifier *= 1.4; }
        modifier = modifier * 1.0 + (self.fuel * 0.1);

        match self.tire {
            Tire::Soft(ref mut age) => {
                let time = 1.0 + (0.066 * *age * (3.52 * self.fuel) / 68.5) + random;
                *age += 1.0 * modifier;
                self.optlap * time
            }
            Tire::Medium(ref mut age) => {
                let time = 1.0 + (0.060 * *age * (3.52 * self.fuel) / 69.0) + random;
                *age += 1.0 * modifier;
                self.optlap * time
            }
            Tire::Hard(ref mut age) => {
                let time = 1.0 + (0.054 * *age * (3.52 * self.fuel) / 69.5) + random;
                *age += 1.0 * modifier;
                self.optlap * time
            }
        }
    }

    // Changed to take &self to access its own methods
    fn check_overtake(&self, deltapace: f32, gap: f32, track: &Track) -> bool {
        let mut rng = rand::thread_rng();
        let exponent = -8.0 * (deltapace - gap / track.overtake);
        let p_overtake = 1.0 / (1.0 + E.powf(exponent));
        rng.gen_range(0.0..=1.0) < p_overtake
    }

    fn gapahead(gap: f32) -> f32 {
        (4.0 * gap + (16.0 * gap.powi(2) + 16.0).sqrt()) / 8.0
    }
}

fn default_mod(track: &Track) -> f32 {
    71.0 / track.laps
}

fn gap_post_overtake() -> f32 {
    let mut rng = rand::thread_rng();
    let normal = Normal::new(0.7, 0.3).unwrap();
    let random: f32 = normal.sample(&mut rng);
    random.max(0.1)
}

#[pyfunction]
fn simulate(mut drivers: Vec<Driver>, track: Track) {
    let d_mod = default_mod(&track);
    let total_laps = track.laps as i32;

    // Loop from current lap to track finish
    for _lap_idx in (drivers[0].lap as i32)..=total_laps {
        let mut laptimes: Vec<f32> = Vec::new();

        // Calculate all laptimes first
        for driver in drivers.iter_mut() {
            laptimes.push(driver.tiremod(d_mod));
        }

        // Handle overtaking and gaps (stopping before the last driver to avoid index out of bounds)
        for i in 0..(drivers.len() - 1) {
            drivers[i].totaltime += laptimes[i];
            
            let gap = drivers[i+1].totaltime - drivers[i].totaltime;
            let deltapace = laptimes[i] - laptimes[i+1];

            if drivers[i].check_overtake(deltapace, gap, &track) {
                // Swap drivers in the list
                drivers.swap(i, i + 1);
                drivers[i].totaltime = drivers[i+1].totaltime + gap_post_overtake();
            } else {
                // If the gap is small, they might be "stuck"
                let new_gap = Driver::gapahead(gap);
                drivers[i].totaltime = drivers[i+1].totaltime + new_gap;
            }
        }
    }
}

#[pymodule]
fn f1strat(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<Tire>()?;
    m.add_function(wrap_pyfunction!(simulate, m)?)?;
    Ok(())
}
