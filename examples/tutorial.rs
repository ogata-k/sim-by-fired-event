use rand::{thread_rng, Rng};
use sim_by_fired_event::event::{Event, EventScheduler, EventTimer, Priority};
use sim_by_fired_event::model::{Model, StepEachEvent};
use sim_by_fired_event::Simulator;
use std::collections::HashMap;
use std::iter::FromIterator;
use std::thread::sleep;
use std::time::{Duration, SystemTime};

/// This tutorial example is timeline simulation

// Create Model structure
#[derive(Debug, Eq, PartialEq, Ord, Clone)]
struct TimelineItem {
    account: String,
    message: String,
    created_at: Duration,
}

impl PartialOrd<TimelineItem> for TimelineItem {
    fn partial_cmp(&self, other: &TimelineItem) -> Option<std::cmp::Ordering> {
        (&self.created_at, &self.account, &self.message).partial_cmp(&(
            &other.created_at,
            &other.account,
            &other.message,
        ))
    }
}

impl std::fmt::Display for TimelineItem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}\n\t{}\n{:#.2?}",
            self.account, self.message, self.created_at
        )
    }
}

#[derive(Debug, Clone)]
struct Timeline {
    // meta
    start: SystemTime,
    follow: HashMap<String, Vec<String>>,
    use_flush: bool,

    // items
    items: Vec<TimelineItem>,
    before_flush: Vec<TimelineItem>,
}

impl Timeline {
    fn new() -> Self {
        Timeline {
            start: SystemTime::now(),
            follow: HashMap::from_iter(
                vec![
                    (
                        "Azio".to_string(),
                        vec![
                            "こんにちは".to_string(),
                            "你好".to_string(),
                            "안녕하세요".to_string(),
                            "Xin chào".to_string(),
                            "नमस्ते".to_string(),
                        ],
                    ),
                    (
                        "Mezoriento".to_string(),
                        vec![
                            "Mezoriento".to_string(),
                            "Merhaba".to_string(),
                            "გამარჯობა".to_string(),
                            "سلام علیکم".to_string(),
                            "ԲարեՎ".to_string(),
                        ],
                    ),
                    (
                        "Eŭropo".to_string(),
                        vec![
                            "Bonjour".to_string(),
                            "Guten tag".to_string(),
                            "Buon giorno".to_string(),
                            "Buon giorno".to_string(),
                            "Olá".to_string(),
                        ],
                    ),
                ]
                .into_iter(),
            ),
            use_flush: false,
            items: vec![],
            before_flush: vec![],
        }
    }

    fn spawn_item(&mut self, account: String, message: String) {
        self.before_flush.push(TimelineItem {
            account,
            message,
            created_at: SystemTime::now().duration_since(self.start).unwrap(),
        });
    }

    fn schedule<R: Rng + ?Sized>(
        &mut self,
        rng: &mut R,
        scheduler: &mut EventScheduler<TimelineEvent>,
        account: &str,
    ) {
        let messages = self.follow.get_mut(account).unwrap();
        messages.rotate_left(1);
        scheduler
            .timeout(
                rng,
                EventTimer::Uniform(20, 30, true),
                Priority::MIN + 1,
                TimelineEvent::Spawn(account.to_string(), messages.first().unwrap().to_string()),
            )
            .unwrap();
    }

    fn flush(&mut self, recorder: &mut Recorder) {
        self.use_flush = false;
        // insert with order by asc
        self.before_flush.sort();
        for item in self.before_flush.drain(..) {
            recorder.record(&item);
            self.items.push(item);
        }
    }
}

// Create Event structure
#[derive(Debug, Clone)]
enum TimelineEvent {
    Flush,
    // pair of account and message
    Spawn(String, String),
}

impl Event for TimelineEvent {}

// Make the events available in the model
struct Recorder {}

impl Recorder {
    fn record(&mut self, item: &TimelineItem) {
        println!("\n{}", item);
    }
}

impl Model<Recorder> for Timeline {
    type ModelEvent = TimelineEvent;

    fn initialize<R: Rng + ?Sized>(
        &mut self,
        rng: &mut R,
        _recorder: &mut Recorder,
        scheduler: &mut EventScheduler<Self::ModelEvent>,
    ) {
        // schedule pull-to-refresh event every 10 frame as the lowest priority
        scheduler
            .every_interval(
                rng,
                EventTimer::Time(10),
                Priority::MIN,
                TimelineEvent::Flush,
            )
            .unwrap();

        // create random post for each account
        let accounts: Vec<String> = self.follow.keys().map(|s| s.to_string()).collect();
        for account in accounts.iter() {
            self.schedule(rng, scheduler, account);
        }
    }

    fn start_frame(&mut self, _recorder: &mut Recorder) {
        // wait time for apply every 0.25sec. (not need)
        sleep(Duration::from_millis(250));
    }

    fn finish_frame(&mut self, recorder: &mut Recorder) {
        if self.use_flush {
            self.flush(recorder);
        }
    }
}

// and impl step
impl StepEachEvent<Recorder, TimelineEvent> for Timeline {
    fn step_each_event<R: Rng + ?Sized>(
        &mut self,
        rng: &mut R,
        _recorder: &mut Recorder,
        scheduler: &mut EventScheduler<Self::ModelEvent>,
        _priority: Priority,
        fired_event: Self::ModelEvent,
    ) {
        match fired_event {
            TimelineEvent::Flush => self.use_flush = true,
            TimelineEvent::Spawn(account, message) => {
                self.schedule(rng, scheduler, account.as_str());
                self.spawn_item(account, message);
            }
        }
    }
}

// Run simulation
fn main() {
    const COUNT: usize = 500;
    let mut rng = thread_rng();
    let model = Timeline::new();
    let mut simulator = Simulator::create_from(&mut rng, model, Recorder {});
    simulator.run_n_each_event(&mut rng, COUNT);
}
