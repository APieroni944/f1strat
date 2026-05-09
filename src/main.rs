use rand::thread_rng;
use rand_distr::{Distribution, Normal};

enum Tire {
    Soft,
    Medium,
    Hard
}
struct Track {
    laps: u8,
    length: f32,
    overtake: f32,
    pit: u8,
}

struct Driver {
    number: u8,
    lap: u8,
    tire: Tire,
    tirelap: f32,
    fuel: f32,
    isstuck: bool,
    totaltime: f32,
    optlap: f32,
}
impl Driver {
    fn tiremod(&mut self, modifier :f32) -> f32 { //Wear based on Red Bull Ring data. Adjustmodifier for track length
        let mut rng = thread_rng();
        let normal = Normal::new(0.0, 0.3).unwrap();
        let random :f32 = normal.sample(&mut rng);
        match self.tire {
            Tire::Soft => {
                let time = 1.0 + 0.070*self.tirelap*(3.52*self.fuel)/68.2 + random;
                self.tirelap += 1.0*modifier;
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
    fn wearmod(&self, defaultmod: f32) -> f32 {
        let mut modifier = defaultmod;
        if self.isstuck { modifier = modifier*1.4;}
        modifier = modifier * 1.0 + (self.fuel * 0.1);
        return modifier;
    }
}

struct Strat{
    tire: Tire,
    target: u8,
    sd: f32,
}

fn DefaultMod(laps: f32) -> f32 {
    71.0/laps
}

fn main() {
    println!("Hello, world!");
}
