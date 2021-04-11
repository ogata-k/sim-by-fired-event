# sim-by-fired-event
This library is the library of discrete time simulation with event scheduling and firing events at the time.

# How to use
use example [timeline](./examples/tutorial.rs).

1. [Create Model structure](#create-model-structure)
2. [Create Event structure](#create-event-structure)
3. [Make the events available in the model and impl step](#make-the-events-available-in-the-model-and-impl-step)
4. [Run simulation](#run-simulation)

## Create Model structure
First, create model structure.
This timeline simulation use the model, as read timeline items lazy.

Following definitions:

```
#[derive(Debug, Eq, PartialEq, Ord, Clone)]
struct TimelineItem {
    account: String,
    message: String,
    created_at: Duration,
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
```

And create follow accounts and the account's messages in model's initializer.

```
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
```

## create Event structure
Second, create event structure.

If do not use event, you can skip this step and so on. Please refer to the [counter](./examples/counter.rs) simulation for details.
Following for the simulation when use event.

I want to use the event which the account request to fetch requested items,
and the event which the account request to create timeline item for.

Following definitions:

```
#[derive(Debug, Clone)]
enum TimelineEvent {
    Flush,
    // pair of account and message
    Spawn(String, String),
}

impl Event for TimelineEvent {}
```

## Make the events available in the model and impl step
Third, make the events available in the model.

We created the model and the events before, But impl the way that the model handle the events.

Following implementations:
(Following the timeline user request refresh every 10 frames.)

```
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
```

This Recorder in over implementations is structure which record log and summary and so on.

The method of initialize is the action in initialize simulation.
The start_frame(finish_frame) is the action in a frame start(end).

These implementations are not implementation the way hot to that the model handle the events.
Because handler's definitions has variety, the implementation is defined in extends trait.
Following implementation is handle each event. If you want to handle whole fired events together, use BulkEvents trait.

```
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
```

## Run simulation
At last, run simulation.

Initialize simulation.

```
let mut rng = thread_rng();
let model = Timeline::new();
let mut simulator = Simulator::create_from(&mut rng, model, Recorder {});
```

Run simulate for COUNT frames.

```
simulator.run_n_each_event(&mut rng, COUNT);
```

If you use simulate other ways (e.g. run one frame, run simulate until .., run with check and update model state),
you can use other run_XXX method.
