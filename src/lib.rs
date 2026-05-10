use rand::Rng;
use rand_distr::{Distribution, Normal};
use std::f32::consts::E;
use pyo3::prelude::*;

#[pyclass]
#[derive(Clone)]
enum Tire {
    Soft,
    Medium,
    Hard
}
#[derive(FromPyObject)]
struct Track {
    laps: f32,
    length: f32,
    overtake: f32,
    pit: f32,
}

#[derive(FromPyObject)]
struct Driver {
    number: u8,
    lap: f32,
    tire: Tire,
    tirelap: f32,
    fuel: f32,
    isstuck: bool,
    totaltime: f32,
    optlap: f32,
    strategy: Vec<Strat>
}
impl Driver {
    fn tiremod(&mut self, defaultmod :f32) -> f32 { //Wear based on Red Bull Ring data. Adjust modifier for track length
        let mut rng = rand::thread_rng();
        let normal = Normal::new(0.0, 0.3).unwrap();
        let random :f32 = normal.sample(&mut rng);

        let mut modifier = defaultmod;
        if self.isstuck { modifier = modifier*1.4;}             //adjust wear modifier for isstuck
        modifier = modifier * 1.0 + (self.fuel * 0.1);          //adjust wear modifier for fuel

        match self.tire {
            Tire::Soft => {
                let time = 1.0 + 0.070*self.tirelap*(3.52*self.fuel)/68.2 + random;     //calculae percentage of optimal time
                self.tirelap += 1.0*modifier;                                           //Apply tire wear
                return time;
            }
            Tire::Medium => {
                let time = 1.0 + 0.060*self.tirelap*(3.52*self.fuel)/69.0 + random;
                self.tirelap += 1.0*modifier;
                return time;
            }
            Tire::Hard => {
                let time = 1.0 + 0.054*self.tirelap*(3.52*self.fuel)/69.5 + random;
                self.tirelap += 1.0*modifier;
                return time;
            }
        }
    }
    fn overtake (deltapace: f32, gap: f32, track: &Track) -> bool {
        let mut rng = rand::thread_rng();
        let e = 2.7182817;
        let exponent = -8.0 * ( deltapace - gap / track.overtake);
        let P_overtake = 1.0 / ( 1.0 + E.powf(exponent));

        rng.gen_range(0.0..=1.0) < P_overtake 
    }
}

#[derive(FromPyObject)]
struct Strat{
    tire: Tire,
    target: f32,
    sd: f32,
}

fn DefaultMod(track: Track) -> f32 {
    71.0/track.laps
}

#[pyfunction]
fn simulate(drivers :Vec<Driver>, track :Track) {
    println!("Hello World");
}

#[pymodule]
fn f1strat(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<Tire>()?; // You must register the enum class
    m.add_function(wrap_pyfunction!(run_f1_sim, m)?)?;
    Ok(())
}

