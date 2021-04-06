use rand::distributions::Distribution;
use rand::distributions::Uniform;
use rand::{thread_rng, Rng};
use sim_rs::event::{Event, EventScheduler, EventTimer, Schedule};
use sim_rs::model::Model;
use sim_rs::SimRs;
use std::collections::BTreeMap;

const FRAME_COUNT: u64 = 100;
const INITIAL_POSITION: (f64, f64) = (0.0, 0.0);

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
enum WalkDirection {
    Stay,
    Up,
    Down,
    Right,
    Left,
    UpRight,
    UpLeft,
    DownRight,
    DownLeft,
}

impl WalkDirection {
    fn create_as_random<R: Rng + ?Sized>(rng: &mut R) -> WalkDirection {
        let directions = Self::get_all_direction();
        let dist = Uniform::from(0_u8..directions.len() as u8);
        *directions.get(dist.sample(rng) as usize).unwrap()
    }

    fn get_all_direction() -> Vec<WalkDirection> {
        use WalkDirection::*;
        vec![
            Stay, Up, Down, Right, Left, UpRight, UpLeft, DownRight, DownLeft,
        ]
    }

    fn get_diff(&self) -> (f64, f64) {
        use WalkDirection::*;
        match self {
            Stay => (0., 0.),
            Up => (0., 1.),
            Down => (0., -1.),
            Right => (1., 0.),
            Left => (-1., 0.),
            UpRight => (1. / 1.41, 1. / 1.41),
            UpLeft => (-1. / 1.41, 1. / 1.41),
            DownRight => (1. / 1.41, -1. / 1.41),
            DownLeft => (-1. / 1.41, -1. / 1.41),
        }
    }
}

// Event
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
struct Walk {
    index: usize,
}

impl Event for Walk {}

#[derive(Debug, Clone)]
struct Walker {
    name: String,
    position: (f64, f64),
    pattern: Schedule,
}

impl Walker {
    fn create_from(schedule: Schedule) -> Self {
        match &schedule {
            Schedule::Immediate => Walker {
                name: "immediate".to_string(),
                position: INITIAL_POSITION,
                pattern: Schedule::Immediate,
            },
            Schedule::Timeout(timer) => Walker {
                name: format!("timeout_{:?}", &timer),
                position: INITIAL_POSITION,
                pattern: schedule,
            },
            Schedule::Everytime => Walker {
                name: format!("everytime"),
                position: INITIAL_POSITION,
                pattern: schedule,
            },
            Schedule::EveryInterval(timer) => Walker {
                name: format!("interval_{:?}", &timer),
                position: INITIAL_POSITION,
                pattern: schedule,
            },
            Schedule::Repeat(count, timer) => Walker {
                name: format!("repeat_{}_{:?}", count, &timer),
                position: INITIAL_POSITION,
                pattern: schedule,
            },
        }
    }

    fn walk(&mut self, direction: WalkDirection) {
        let current = self.position;
        let diff = direction.get_diff();
        self.position = (current.0 + diff.0, current.1 + diff.1);
    }

    fn schedule<R: Rng + ?Sized>(
        &self,
        rng: &mut R,
        scheduler: &mut EventScheduler<Walk>,
        index: usize,
    ) {
        let schedule = self.pattern.clone();
        let _ = scheduler.schedule(rng, schedule, Walk { index });
    }
}

// model
struct WalkerList {
    timer: u64,
    walkers: Vec<Walker>,
}

fn get_all_patterns() -> Vec<Schedule> {
    use Schedule::*;
    let mut result = vec![];

    result.push(Immediate);
    result.push(Everytime);
    for (index, timer) in vec![
        EventTimer::Time(10),
        EventTimer::Uniform(1..10),
        EventTimer::WeightedIndex(vec![(1, 2), (5, 5), (10, 2)]),
    ]
    .iter()
    .enumerate()
    {
        result.push(Timeout(timer.clone()));
        result.push(EveryInterval(timer.clone()));
        result.push(Repeat(((index + 1) * 3) as u8, timer.clone()));
    }

    result
}

type Recorder = BTreeMap<usize, Vec<(u64, WalkDirection)>>;
impl Model<Recorder> for WalkerList {
    type ModelEvent = Walk;

    fn initialize<R: Rng + ?Sized>(&mut self, _rng: &mut R, recorder: &mut Recorder) {
        self.timer = 0;
        self.walkers.clear();
        recorder.clear();

        let patterns: Vec<Schedule> = get_all_patterns();
        for schedule in patterns.into_iter() {
            let walker = Walker::create_from(schedule);
            self.walkers.push(walker);
        }
    }

    fn initialize_frame<R: Rng + ?Sized>(
        &mut self,
        rng: &mut R,
        _recorder: &mut Recorder,
        scheduler: &mut EventScheduler<Self::ModelEvent>,
    ) {
        print!("Walker:");
        for (index, walker) in self.walkers.iter().enumerate() {
            print!("\t{}", walker.name);
            walker.schedule(rng, scheduler, index);
        }
        println!();

        self.print_walk_state();
    }

    fn step<R: Rng + ?Sized>(
        &mut self,
        rng: &mut R,
        recorder: &mut Recorder,
        _scheduler: &mut EventScheduler<Self::ModelEvent>,
        fired_events: &mut Vec<Self::ModelEvent>,
    ) {
        self.timer += 1;
        print!("fired:");
        for fired in fired_events.iter() {
            let index = fired.index;
            let walker = self.walkers.get_mut(index).unwrap();
            let direction = WalkDirection::create_as_random(rng);
            print!("\t{}:{:?}", index, direction);
            recorder
                .entry(index)
                .or_default()
                .push((self.timer, direction));
            walker.walk(direction);
        }
        println!();

        self.print_walk_state();
    }
}

impl WalkerList {
    fn new() -> Self {
        WalkerList {
            timer: 0,
            walkers: vec![],
        }
    }

    fn print_walk_state(&self) {
        print!("at {}:", self.timer);
        for walker in self.walkers.iter() {
            print!("\t({:.2}, {:.2})", walker.position.0, walker.position.1);
        }
        println!();
    }
}

fn main() {
    let model = WalkerList::new();
    let mut simulator = SimRs::create_from(model, Default::default());
    let mut rng = thread_rng();
    simulator.initialize(&mut rng);
    simulator.run(&mut rng, FRAME_COUNT);

    println!();
    for (index, logs) in simulator.get_recorder() {
        println!("{}: {:?}", index, logs);
    }
}
