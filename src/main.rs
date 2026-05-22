use rand::Rng;
use rand_distr::{Distribution, Normal};
use std::f32::consts::E;

struct Driver {
    number: u32,
    lap: u32,
    tire: (u32, f32),           //using tuple for simpler python integration than enum (compound, lap)
    fuel: f32,
    totaltime: f32,
    optlap: f32,
    isstuck: bool,
    strat: Vec<Strategy>,
}

struct Track {
    laps: u32,
    overtake: f32,
    pitloss: f32,
}

struct Strategy {
    tire: u32,
    lap: u32,
    sd: f32,
}

impl Driver {
    fn tiremod(&mut self, defaultmod: f32, rng: &mut impl Rng) -> f32 {

        self.lap += 1;

        let normal = Normal::new(0.0, 0.3).unwrap();
        let random: f32 = normal.sample(rng);

        let mut modifier = defaultmod;
        if self.isstuck { modifier *= 1.4; }
        modifier += self.fuel * 0.1;

        match self.tire.0 {
            0 => {
                let time = 1.0 + (0.066 * self.tire.1 * (3.52 * self.fuel) / 68.5) + random;
                self.tire.1 += 1.0 * modifier;
                self.optlap * time
            }
            1 => {
                let time = 1.0 + (0.060 * self.tire.1 * (3.52 * self.fuel) / 69.0) + random;
                self.tire.1 += 1.0 * modifier;
                self.optlap * time
            }
            2 => {
                let time = 1.0 + (0.054 * self.tire.1 * (3.52 * self.fuel) / 69.5) + random;
                self.tire.1 += 1.0 * modifier;
                self.optlap * time
            }
            _ => self.optlap,
        }
    }
}

impl Track {
    fn findDefaultMod(&self) -> f32 {
        71 as f32 /self.laps as f32
    }
}

fn AdjustGap(drivers: &mut [Driver]) {
    let mut gaps = [0.0; 32];                                               //oversized array for future reusability
    for i in (1..drivers.len()) {
        let gap = drivers[i].totaltime - drivers[i-1].totaltime;
        gaps[i] = (4.0 * gap + (16.0 * gap.powi(2) + 16.0).sqrt()) / 8.0;
    }
    for i in (1..drivers.len()) {
        drivers[i].totaltime = drivers[i-1].totaltime + gaps[i];
        drivers[i].isstuck = gaps[i] < 1.2;
    }
}

fn CheckOvertake(drivers: &mut [Driver], track: &Track, laptimes: &mut [f32], rng: &mut impl Rng) {
    let normal = Normal::new(0.7, 0.3).unwrap();
    for i in (1..drivers.len()) {
        let gap = drivers[i].totaltime - drivers[i-1].totaltime;
        let deltapace = laptimes[i] - laptimes [i-1];
        let exponent = -8.0 * (deltapace - gap / track.overtake);
        let p_overtake = 1.0 / (1.0 + E.powf(exponent));
        if rng.gen_range(0.0..=1.0) < p_overtake {
            drivers.swap(i, i-1);
            laptimes.swap(i, i-1);
            let gap: f32 = (normal.sample(rng) as f32).max(0.1);
            drivers[i].totaltime = drivers[i-1].totaltime + gap;
        }
    }
}

fn CheckPitstop(drivers: &mut [Driver], track: &Track, rng: &mut impl Rng) {
    for i in (0..drivers.len()) {

        if drivers[i].strat.is_empty() {continue; }


        let k = 1.8138 / drivers[i].strat[0].sd;                          //pit lap flexibility
        let mut x = drivers[i].lap as f32 - drivers[i].strat[0].lap as f32;         //distance from target lap

        if drivers[i].isstuck{ x += 2.0; }
        if i < drivers.len() - 1 {
            if drivers[i+1].isstuck{ x -= 2.0; }
        }

        if rng.gen_range(0.0..=1.0) < 1.0 / (1.0 + (-k * x).exp()) {
            drivers[i].totaltime += track.pitloss;
            drivers[i].tire = (drivers[i].strat[0].tire, 0.0);
            drivers[i].strat.remove(0); 
        }
        let mut j = i;
        while j < drivers.len() - 1 && drivers[j].totaltime < drivers[j+1].totaltime {
            drivers.swap(j, j+1);
            j += 1;
        }
    }
}

fn SimulateLap(drivers: &mut [Driver], track: &Track, rng: &mut impl Rng, defaultmod: f32) {
    let mut laptime: [f32; 22] = std::array::from_fn(|i| drivers[i].tiremod(defaultmod, rng));
    for i in (0..drivers.len()) {drivers[i].totaltime += laptime[i];}
    CheckOvertake(drivers, track, &mut laptime, rng);
    AdjustGap(drivers);
    CheckPitstop(drivers, track, rng);
}

fn main() {
    println!("Hello world!")
}
