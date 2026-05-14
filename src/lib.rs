use rand::Rng;
use rand_distr::{Distribution, Normal};
use std::f32::consts::E;
use pyo3::prelude::*;

#[pyclass]
#[derive(Clone, Copy, Debug)]
pub enum Tire {
    Soft(f32),
    Medium(f32),
    Hard(f32),
}

#[pyclass]
#[derive(Clone, Copy, Debug)]
pub struct Track {
    #[pyo3(get, set)]
    pub laps: f32,
    #[pyo3(get, set)]
    pub length: f32,
    #[pyo3(get, set)]
    pub overtake: f32,
    #[pyo3(get, set)]
    pub pit: f32,
    #[pyo3(get, set)]
    pub ideallap: f32,
}

#[pymethods]
impl Track {
    #[new]
    fn new(laps: f32, length: f32, overtake: f32, pit: f32, ideallap: f32) -> Self {
        Track { laps, length, overtake, pit, ideallap }
    }
}

#[pyclass]
#[derive(Clone, Debug)]
pub struct Strat {
    #[pyo3(get, set)]
    pub tire: Tire,
    #[pyo3(get, set)]
    pub target: f32,
    #[pyo3(get, set)]
    pub sd: f32,
}

#[pymethods]
impl Strat {
    #[new]
    fn new(tire: Tire, target: f32, sd: f32) -> Self {
        Strat { tire, target, sd }
    }
}

#[pyclass]
#[derive(Clone, Debug)]
pub struct Driver {
    #[pyo3(get, set)]
    pub number: u8,
    #[pyo3(get, set)]
    pub lap: f32,
    #[pyo3(get, set)]
    pub tire: Tire,
    #[pyo3(get, set)]
    pub fuel: f32,
    #[pyo3(get, set)]
    pub totaltime: f32,
    #[pyo3(get, set)]
    pub optlap: f32,
    #[pyo3(get, set)]
    pub isstuck: bool,
    #[pyo3(get, set)]
    pub strategy: Vec<Strat>,
}

#[pymethods]
impl Driver {
    #[new]
    fn new(number: u8, lap: f32, tire: Tire, fuel: f32, totaltime: f32, optlap: f32, isstuck: bool, strategy: Vec<Strat>) -> Self {
        Driver { number, lap, tire, fuel, totaltime, optlap, isstuck, strategy }
    }
}

// Internal Rust processing logic
impl Driver {
    fn tiremod(&mut self, defaultmod: f32) -> f32 {
        let mut rng = rand::thread_rng();
        let normal = Normal::new(0.0, 0.3).unwrap();
        let random: f32 = normal.sample(&mut rng);

        let mut modifier = defaultmod;
        if self.isstuck { modifier *= 1.4; }
        modifier += self.fuel * 0.1;

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

    fn pitstop(&mut self, car_behind: bool, track: &Track) -> bool {
        if self.strategy.is_empty() { return false; }
        let mut rng = rand::thread_rng();
        let k = 1.8138 / self.strategy[0].sd.max(0.1);
        let mut x = match self.tire {
            Tire::Soft(a) | Tire::Medium(a) | Tire::Hard(a) => a
        } - self.strategy[0].target;
        
        if self.isstuck { x += 2.0; }
        if car_behind { x -= 2.0; }

        let p_pit = 1.0 / (1.0 + (-k * x).exp());
        let will_pit = rng.gen_range(0.0..=1.0) < p_pit;

        if will_pit {
            self.totaltime += track.pit;
            self.strategy.remove(0); 
            if let Some(next_strat) = self.strategy.get_mut(0) {
                self.tire = next_strat.tire;
                next_strat.target -= 0.5 * x;
            }
        }
        will_pit
    }
}

#[pyfunction]
pub fn simulate(mut drivers: Vec<Driver>, track: Track) -> PyResult<Vec<Driver>> {
    let d_mod = 71.0 / track.laps;
    for _ in 0..(track.laps as i32) {
        let mut laptimes = Vec::new();
        for d in drivers.iter_mut() {
            laptimes.push(d.tiremod(d_mod));
        }
        
        let mut pit_occurred = false;
        for i in 0..drivers.len() {
            let behind = if i > 0 { drivers[i-1].isstuck } else { false };
            if drivers[i].pitstop(behind, &track) { pit_occurred = true; }
        }

        if pit_occurred {
            drivers.sort_by(|a, b| a.totaltime.partial_cmp(&b.totaltime).unwrap());
        }

        for i in 0..drivers.len() {
            drivers[i].totaltime += laptimes[i];
            drivers[i].fuel -= 0.3;
        }

        // Simplistic Overtake resolution
        for i in 0..(drivers.len() - 1) {
            let gap = drivers[i+1].totaltime - drivers[i].totaltime;
            if gap < 0.1 { // Over-simplified check for example
                drivers.swap(i, i+1);
            }
            drivers[i].isstuck = gap <= 1.0;
        }
    }
    Ok(drivers)
}

#[pymodule]
fn f1strat(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<Tire>()?;
    m.add_class::<Track>()?;
    m.add_class::<Strat>()?;
    m.add_class::<Driver>()?;
    m.add_function(wrap_pyfunction!(simulate, m)?)?;
    Ok(())
}
