use rand::Rng;
use rand_distr::{Distribution, Normal};
use std::f32::consts::E;
use pyo3::prelude::*;

#[pyclass]
#[derive(Clone)]
enum Tire {
    Soft(f32),
    Medium(f32),
    Hard(f32),
}
#[derive(FromPyObject)]
struct Track {
    laps: f32,
    length: f32,
    overtake: f32,
    pit: f32,
    ideallap: f32,
}

#[derive(FromPyObject)]
struct Driver {
    number: u8,
    lap: f32,
    tire: Tire,
    fuel: f32,
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
            Tire::Soft(ref mut age) => {
                let time = 1.0 + 0.070*age*(3.52*self.fuel)/68.2 + random;     //calculae percentage of optimal time
                *age += 1.0*modifier;                                           //Apply tire wear
                return self.optlap * time;
            }
            Tire::Medium(ref mut age) => {
                let time = 1.0 + 0.060*age*(3.52*self.fuel)/69.0 + random;
                *age += 1.0*modifier;
                return self.optlap * time;
            }
            Tire::Hard(ref mut age) => {
                let time = 1.0 + 0.054*age*(3.52*self.fuel)/69.5 + random;
                *age += 1.0*modifier;
                return self.optlap * time;
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
    fn gapahead (gap: f32) -> f32 {
        //let x = gap - deltapace;
        (4.0 * gap + (16.0 * gap.powi(2) + 16.0).sqrt()) / (2.0 * 4.0)
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
fn gapPostOvertake() -> f32 {
    let mut rng = rand::thread_rng();
    let normal = Normal::new(0.7, 0.3).unwrap();
    let random :f32 = normal.sample(&mut rng);
    if random > 0.1 {
        return random;
    } else {
        return 0.1
    }
}

#[pyfunction]
fn simulate(mut drivers :Vec<Driver>, track :Track) {
    let defaultmod :f32 = DefaultMod(track);
    for lap in &driver[0].lap..=track.laps {
        let mut laptime: Vec<f32> = Vec::new; 
        for i in (0..=drivers.len()) {
            laptime.push(drivers[i].tiremod(defaultmod))
            driver[i].totaltime += laptime[i]

            
            let deltapace = laptime[i] - laptime[i+1]
            if drivers[i].overtake(deltapace, gap, track) {
                let temp = driver [i];
                driver[i] = driver[i+1];
                driver[i+1] = temp;
                driver[i].totaltime = driver[i+1].totaltime + gapPostOvertake();
            } else {
                let gap = match drivers[i+1].totaltime - drivers[i].totaltime {
                    Ok(gap) => gapahead(gap),
                    err(_) => continue,
                };
                drivers[i].totaltime = drivers[i+1].totaltime + gap;
            }
        }
    }
}

#[pymodule]
fn f1strat(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<Tire>()?; // You must register the enum class
    m.add_function(wrap_pyfunction!(simulate, m)?)?;
    Ok(())
}

