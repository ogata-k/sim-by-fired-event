use sim_by_fired_event::model::NothingEventModel;
use sim_by_fired_event::NothingEventSimulator;

#[derive(Debug, Default, Clone)]
struct Counter {
    count: usize,
}

#[derive(Debug, Default, Clone)]
struct Recorder {}

impl Recorder {
    fn record(&mut self, count: &usize) {
        println!("count: {}", count);
    }
}

impl NothingEventModel<Recorder> for Counter {
    fn initialize(&mut self, recorder: &mut Recorder) {
        recorder.record(&self.count);
    }

    fn start_frame(&mut self, _recorder: &mut Recorder) {
        // none
    }

    fn step(&mut self, _recorder: &mut Recorder) {
        self.count += 1;
    }

    fn finish_frame(&mut self, recorder: &mut Recorder) {
        recorder.record(&self.count);
    }
}

fn main() {
    const COUNT: usize = 10;
    let mut sim = NothingEventSimulator::create_from(Counter::default(), Recorder::default());
    sim.run_n(COUNT);
}
