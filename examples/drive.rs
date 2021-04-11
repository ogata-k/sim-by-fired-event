use rand::{thread_rng, Rng};
use sim_by_fired_event::event::{Event, EventScheduler, EventTimer, Priority};
use sim_by_fired_event::model::{BulkEvents, Model};
use sim_by_fired_event::Simulator;

#[derive(Debug, Eq, PartialEq, Copy, Clone)]
enum CarStatus {
    Driving,
    Charge,
    EngineStop,
}

#[derive(Debug, Eq, PartialEq, Copy, Clone)]
struct Car {
    fuel: u16,
    status: CarStatus,
}

#[derive(Debug, Default, Eq, PartialEq, Copy, Clone)]
struct CarRecorder {
    time: u16,
    use_fuel: u16,
    total_inject_fuel: u16,
    total_run: u16,
}

impl CarRecorder {
    fn start_next_time(&mut self) {
        self.time += 1;
    }

    fn add_use_fuel(&mut self, fuel: u16) {
        self.use_fuel += fuel;
    }

    fn add_inject_fuel(&mut self, fuel: u16) {
        self.total_inject_fuel += fuel;
    }

    fn add_run(&mut self, running: u16) {
        self.total_run += running;
    }
}

impl Model<CarRecorder> for Car {
    type ModelEvent = CarEvent;

    fn initialize<R: Rng + ?Sized>(
        &mut self,
        rng: &mut R,
        _recorder: &mut CarRecorder,
        scheduler: &mut EventScheduler<CarEvent>,
    ) {
        println!("ride on the car");
        let _ = scheduler.timeout(
            rng,
            EventTimer::WeightedIndex(vec![(5, 3), (10, 2), (15, 1)]),
            0,
            CarEvent::StartCharge,
        );
    }

    fn start_frame(&mut self, _recorder: &mut CarRecorder) {
        // none
    }

    fn finish_frame(&mut self, _recorder: &mut CarRecorder) {
        // none
    }
}

impl BulkEvents<CarRecorder, CarEvent> for Car {
    fn step_in_bulk<R: Rng + ?Sized>(
        &mut self,
        rng: &mut R,
        recorder: &mut CarRecorder,
        scheduler: &mut EventScheduler<CarEvent>,
        fired_events: Vec<(Priority, CarEvent)>,
    ) {
        if self.status == CarStatus::EngineStop {
            return;
        }

        recorder.start_next_time();
        // fired event is always fired at most one.
        if let Some(event) = fired_events.iter().map(|(_, fired)| fired).nth(0) {
            match event {
                CarEvent::StartCharge => {
                    println!("go to gas station");
                    self.status = CarStatus::Charge;
                    let _ = scheduler.timeout(
                        rng,
                        EventTimer::WeightedIndex(vec![(2, 3), (3, 2), (5, 1)]),
                        0,
                        CarEvent::EndCharge,
                    );
                }
                CarEvent::EndCharge => {
                    println!("leave gas station");
                    self.status = CarStatus::Driving;
                    let _ = scheduler.timeout(
                        rng,
                        EventTimer::WeightedIndex(vec![(5, 3), (10, 2), (15, 1)]),
                        0,
                        CarEvent::StartCharge,
                    );
                }
            }
        } else {
            if self.status == CarStatus::Charge {
                println!("charge fuel");
                self.charge(recorder);

                if self.fuel == Self::MAX_FUEL {
                    scheduler.clear();
                    let _ = scheduler.immediate(rng, 0, CarEvent::EndCharge);
                }
            } else if self.status == CarStatus::Driving {
                println!("drive the car");
                self.drive(recorder);

                if self.fuel == 0 {
                    println!("Ops! stop engine!!");
                    self.status = CarStatus::EngineStop;
                    scheduler.clear();
                }
            }
        }
    }
}

impl Car {
    const MAX_FUEL: u16 = 20;
    const ADD_FUEL_PER_TIME: u16 = 2;
    const USE_FUEL_PER_TIME: u16 = 1;
    const CAN_RUNNING_PER_FUEL: u16 = 16;

    fn new() -> Self {
        Car {
            fuel: Self::MAX_FUEL,
            status: CarStatus::Driving,
        }
    }

    fn drive(&mut self, recorder: &mut CarRecorder) {
        if self.fuel > 0 {
            self.fuel -= Self::USE_FUEL_PER_TIME;
            recorder.add_use_fuel(Self::USE_FUEL_PER_TIME);
            recorder.add_run(Self::CAN_RUNNING_PER_FUEL)
        }
    }

    fn charge(&mut self, recorder: &mut CarRecorder) {
        let injected = u16::min(self.fuel + Self::ADD_FUEL_PER_TIME, Self::MAX_FUEL);
        let inject_fuel = injected - self.fuel;
        recorder.add_inject_fuel(inject_fuel);
        self.fuel = injected;
    }
}

#[derive(Debug, PartialOrd, Ord, Eq, PartialEq, Copy, Clone)]
enum CarEvent {
    StartCharge,
    EndCharge,
}

impl Event for CarEvent {}

fn main() {
    let mut rng = thread_rng();
    let model = Car::new();
    let mut simulator = Simulator::create_from(&mut rng, model, CarRecorder::default());
    simulator.run_until_in_bulk_event(&mut rng, |model| model.status != CarStatus::EngineStop);

    println!();
    let recorder = simulator.get_recorder();
    println!("result: {:?}", recorder);
}
