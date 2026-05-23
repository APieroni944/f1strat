use rand::Rng;
use rand_distr::{Distribution, Normal};
use std::f32::consts::E;
use rayon::prelude::*;

#[derive(Clone)]
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

#[derive(Clone)]
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

fn AdjustGap(drivers: &mut [Driver; n]) {
    let mut gaps = [0.0; n];                                               //oversized array for future reusability
    for i in (1..drivers.len()) {
        let gap = drivers[i].totaltime - drivers[i-1].totaltime;
        gaps[i] = (4.0 * gap + (16.0 * gap.powi(2) + 16.0).sqrt()) / 8.0;
    }
    for i in (1..drivers.len()) {
        drivers[i].totaltime = drivers[i-1].totaltime + gaps[i];
        drivers[i].isstuck = gaps[i] < 1.2;
    }
}

fn CheckOvertake(drivers: &mut [Driver; n], track: &Track, laptimes: &mut [f32], rng: &mut impl Rng) {
    let normal = Normal::new(0.7, 0.3).unwrap();
    for i in (1..drivers.len()) {
        let gap = drivers[i].totaltime - drivers[i-1].totaltime;
        let deltapace = laptimes[i-1] - laptimes [i];
        let exponent = -8.0 * (deltapace - gap / track.overtake);
        let p_overtake = 1.0 / (1.0 + E.powf(exponent));
        if rng.gen_range(0.0..=1.0) < p_overtake {
            let gap: f32 = (normal.sample(rng) as f32).max(0.1);
            drivers[i-1].totaltime = drivers[i].totaltime + gap;
            drivers.swap(i, i-1);
            laptimes.swap(i, i-1);
        }
    }
}

fn CheckPitstop(drivers: &mut [Driver; n], track: &Track, rng: &mut impl Rng) {
    for i in (0..drivers.len()) {

        if drivers[i].strat.is_empty() {continue; }


        let k = 1.8138 / drivers[i].strat[0].sd;                          //pit lap flexibility
        let mut x = drivers[i].lap as f32 - drivers[i].strat[0].lap as f32;         //distance from target lap

        if drivers[i].isstuck{ x += 0.0; }
        if i < drivers.len() - 1 {
            if drivers[i+1].isstuck{ x -= 0.0; }
        }

        if rng.gen_range(0.0..=1.0) < 1.0 / (1.0 + (-k * x).exp()) {
            drivers[i].totaltime += track.pitloss;
            drivers[i].tire = (drivers[i].strat[0].tire, 0.0);
            drivers[i].strat.remove(0); 
        }
        let mut j = i;
        while j < drivers.len() - 1 && drivers[j].totaltime > drivers[j+1].totaltime {
            drivers.swap(j, j+1);
            j += 1;
        }
    }
}
fn SortDrivers(drivers: &mut [Driver; n]) {
    loop {
        let mut noswaps = true;
        for i in 0..drivers.len() - 1 {
            if drivers[i].totaltime > drivers[i+1].totaltime {
                drivers.swap(i, i+1);
                noswaps = false;
            }
        }
        if noswaps {break;}
    }
}

fn SimulateLap(drivers: &mut [Driver; n], track: &Track, rng: &mut impl Rng, defaultmod: f32) {
    let mut laptime: [f32; n] = std::array::from_fn(|i| drivers[i].tiremod(defaultmod, rng));
    for i in (0..drivers.len()) {drivers[i].totaltime += laptime[i];}
    CheckOvertake(drivers, track, &mut laptime, rng);
    AdjustGap(drivers);
    CheckPitstop(drivers, track, rng);
    //SortDrivers(drivers);
}

fn SimulateRace(mut drivers: [Driver; n], track: &Track, rng: &mut impl Rng, defaultmod: f32) -> [Driver; n] {
    for i in drivers[0].lap..=track.laps {
        SimulateLap(&mut drivers, track, rng, defaultmod);
    }
    drivers
}

fn SimulateFull(drivers: [Driver; n], track: Track) -> Vec<[Driver; n]>{
    let defaultmod = track.findDefaultMod();
    let result: Vec<[Driver; n]> = (0..10_000).into_par_iter()
        .map(|_i| {
            let mut rng = rand::thread_rng();
            SimulateRace(drivers.clone(), &track, &mut rng, defaultmod)
        })
        .collect();
    return result;
}

const n: usize = 22; 
const N: usize = 22;

fn main() {
    // 1. Initialize Track parameters (e.g., a 50-lap race with standard overtake and pitloss windows)
    let track = Track {
        laps: 55,
        overtake: 4.5,  // Represents track layout passing difficulty modifier
        pitloss: 22.5,   // Time in seconds lost during a standard trip down the pitlane
    };

    // 2. Define standard template strategy paths for different starting compounds
    let soft_strat = vec![
        Strategy { tire: 1, lap: 15, sd: 1.5 }, // Switch to Mediums on lap 15
        Strategy { tire: 2, lap: 35, sd: 2.0 }, // Switch to Hards on lap 35
    ];

    let medium_strat = vec![
        Strategy { tire: 2, lap: 22, sd: 2.5 }, // Switch to Hards on lap 22
    ];

    // 3. Populate a hardcoded array matching your static constraint of exactly N = 22 drivers
    // Grid matches realistic racing numbers, lap tracking intervals, and fuel metrics
    let starting_grid: [Driver; N] = [
// --- Top Tier (Title Contenders) ---
Driver { number: 1,  lap: 0, tire: (0, 0.0), fuel: 80.0, totaltime: 0.0, optlap: 73.0, isstuck: false, strat: get_test_strategy() },
    Driver { number: 2,  lap: 0, tire: (0, 0.0), fuel: 80.0, totaltime: 0.0, optlap: 73.3, isstuck: false, strat: get_test_strategy() },
    Driver { number: 3,  lap: 0, tire: (0, 0.0), fuel: 80.0, totaltime: 0.0, optlap: 73.6, isstuck: false, strat: get_test_strategy() },
    Driver { number: 4,  lap: 0, tire: (0, 0.0), fuel: 80.0, totaltime: 0.0, optlap: 73.9, isstuck: false, strat: get_test_strategy() },
    Driver { number: 5,  lap: 0, tire: (0, 0.0), fuel: 80.0, totaltime: 0.0, optlap: 74.2, isstuck: false, strat: get_test_strategy() },
    Driver { number: 6,  lap: 0, tire: (0, 0.0), fuel: 80.0, totaltime: 0.0, optlap: 74.5, isstuck: false, strat: get_test_strategy() },
    Driver { number: 7,  lap: 0, tire: (0, 0.0), fuel: 80.0, totaltime: 0.0, optlap: 74.8, isstuck: false, strat: get_test_strategy() },
    Driver { number: 8,  lap: 0, tire: (0, 0.0), fuel: 80.0, totaltime: 0.0, optlap: 75.1, isstuck: false, strat: get_test_strategy() },
    Driver { number: 9,  lap: 0, tire: (0, 0.0), fuel: 80.0, totaltime: 0.0, optlap: 75.4, isstuck: false, strat: get_test_strategy() },
    Driver { number: 10, lap: 0, tire: (0, 0.0), fuel: 80.0, totaltime: 0.0, optlap: 75.7, isstuck: false, strat: get_test_strategy() },
    Driver { number: 11, lap: 0, tire: (0, 0.0), fuel: 80.0, totaltime: 0.0, optlap: 76.0, isstuck: false, strat: get_test_strategy() },
    Driver { number: 12, lap: 0, tire: (0, 0.0), fuel: 80.0, totaltime: 0.0, optlap: 76.3, isstuck: false, strat: get_test_strategy() },
    Driver { number: 13, lap: 0, tire: (0, 0.0), fuel: 80.0, totaltime: 0.0, optlap: 76.6, isstuck: false, strat: get_test_strategy() },
    Driver { number: 14, lap: 0, tire: (0, 0.0), fuel: 80.0, totaltime: 0.0, optlap: 76.9, isstuck: false, strat: get_test_strategy() },
    Driver { number: 15, lap: 0, tire: (0, 0.0), fuel: 80.0, totaltime: 0.0, optlap: 77.2, isstuck: false, strat: get_test_strategy() },
    Driver { number: 16, lap: 0, tire: (0, 0.0), fuel: 80.0, totaltime: 0.0, optlap: 77.5, isstuck: false, strat: get_test_strategy() },
    Driver { number: 17, lap: 0, tire: (0, 0.0), fuel: 80.0, totaltime: 0.0, optlap: 77.8, isstuck: false, strat: get_test_strategy() },
    Driver { number: 18, lap: 0, tire: (0, 0.0), fuel: 80.0, totaltime: 0.0, optlap: 78.1, isstuck: false, strat: get_test_strategy() },
    Driver { number: 19, lap: 0, tire: (0, 0.0), fuel: 80.0, totaltime: 0.0, optlap: 78.4, isstuck: false, strat: get_test_strategy() },
    Driver { number: 20, lap: 0, tire: (0, 0.0), fuel: 80.0, totaltime: 0.0, optlap: 78.7, isstuck: false, strat: get_test_strategy() },
    Driver { number: 21, lap: 0, tire: (0, 0.0), fuel: 80.0, totaltime: 0.0, optlap: 79.0, isstuck: false, strat: get_test_strategy() },
    Driver { number: 22, lap: 0, tire: (0, 0.0), fuel: 80.0, totaltime: 0.0, optlap: 79.3, isstuck: false, strat: get_test_strategy() },
    ];
    println!("Executing 10,000 parallel simulation runs...");
    
    // 4. Run the simulation
    let sim_output = SimulateFull(starting_grid.clone(), track);
    
    println!("Simulations complete! Total datasets captured: {}", sim_output.len());
    //println!("First simulated grid sample winner number: {}", sim_output[0][0].number);
    for i in 0..20 {
        println!("simulation {i}: Winner{}, Second{}, Third{}", sim_output[i][0].number, sim_output[i][1].number, sim_output[i][2].number);
    }
    PrintVisualHeatmap(&sim_output, &starting_grid)
}

fn PrintVisualHeatmap(sim_data: &Vec<[Driver; n]>, starting_grid: &[Driver; n]) {
    let matrix = AggregateToMatrixWithRealNumbers(sim_data, starting_grid);
    let driver_lookup: [u32; n] = std::array::from_fn(|i| starting_grid[i].number);

    println!("\n================== FINISHING POSITION HEATMAP ==================");
    println!("Driver | P1                    P5                    P15                  P22");
    println!("-------|---------------------------------------------------------------------");

    for row in 0..n {
        print!(" #{:<3}  | ", driver_lookup[row]);
        for pos in 0..n {
            let prob = matrix[row][pos];
            // Assign a visual shading block based on probability density percentage
            let symbol = match prob {
                p if p > 0.50 => "█", // Heavy favorite cluster (>50% chance)
                p if p > 0.25 => "▓", // Strong chance (25% - 50%)
                p if p > 0.10 => "▒", // Moderate chance (10% - 25%)
                p if p > 0.02 => "░", // Outside chance (2% - 10%)
                _ => "·",             // Near zero chance
            };
            print!("{}  ", symbol); // Print with spacing for visual grid shape
        }
        println!();
    }
    println!("=====================================================================");
}

fn get_test_strategy() -> Vec<Strategy> {
    vec![
        Strategy { tire: 1, lap: 20, sd: 1.0 }, // Lap 20: Switch to Prime (1)
        Strategy { tire: 2, lap: 45, sd: 1.0 }, // Lap 45: Switch to Quali (2)
    ]
}

/// Takes the 10,000 simulated race grids and maps them into a fixed 22x22 matrix.
/// Rows = The original order of drivers in your starting_grid.
/// Columns = Finishing Position index (0 = 1st place, 21 = 22nd place).
fn AggregateToMatrixWithRealNumbers(sim_data: &Vec<[Driver; N]>, starting_grid: &[Driver; N]) -> [[f32; N]; N] {
    let total_races = sim_data.len() as f32;
    
    // 1. Initialise a raw frequency counting grid on the stack filled with zeroes
    let mut raw_counts = [[0u32; N]; N];

    // 2. Extract the exact order of real driver numbers from your starting grid layout
    // Example lookup layout: [1, 11, 16, 55, 63, 44, 4, 81, 14, 18, 10, 31, 23, 2, 22, 3, 77, 24, 20, 27, 21, 30]
    let driver_lookup: [u32; N] = std::array::from_fn(|i| starting_grid[i].number);

    // 3. Loop over every completed race and track finishing placements
    for race_grid in sim_data {
        for (pos, driver) in race_grid.iter().enumerate() {
            
            // Find where this driver's number sits in the original grid lineup.
            // This maps their real racing number to a row index between 0 and 21.
            if let Some(row_index) = driver_lookup.iter().position(|&num| num == driver.number) {
                raw_counts[row_index][pos] += 1;
            }
        }
    }

    // 4. Divide raw counts by the total number of races to get a probability percentage fraction
    let mut probability_matrix = [[0.0f32; N]; N];
    for row in 0..N {
        for pos in 0..N {
            probability_matrix[row][pos] = raw_counts[row][pos] as f32 / total_races;
        }
    }

    probability_matrix
}

