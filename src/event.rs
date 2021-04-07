//! Simulator's event and the event manager

use rand::distributions::{Distribution, Uniform, WeightedError, WeightedIndex};
use rand::Rng;
use std::ops::Range;

/// Timer for local
pub type LocalEventTime = u32;

/// can store event as Simulator's event
pub trait Event: Clone {}

/// helper. if not use event, you can use this.
#[derive(Default, Debug, Ord, PartialOrd, Eq, PartialEq, Copy, Clone)]
pub struct NoneEvent {}
impl Event for NoneEvent {}

/// Error for scheduled event
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScheduleEventError {
    /// user schedule event which scheduler will not fire.
    /// Not occurred in re-schedule event. If occurred at the time, scheduler is panic.
    /// for example, occurred when user schedule repeat count 0 repeat schedule.
    CannotFireEvent,
    WeightedError(WeightedError),
}

impl std::error::Error for ScheduleEventError {}

impl std::fmt::Display for ScheduleEventError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match *self {
            ScheduleEventError::CannotFireEvent => write!(f, "Cannot fire the event"),
            ScheduleEventError::WeightedError(we) => write!(f, "{}", we),
        }
    }
}

impl From<WeightedError> for ScheduleEventError {
    fn from(we: WeightedError) -> Self {
        ScheduleEventError::WeightedError(we)
    }
}

/// timer for schedule
#[derive(Debug, Clone)]
pub enum EventTimer {
    /// fire after timeout
    Time(LocalEventTime),
    /// fire after random value by uniform select in range values
    Uniform(Range<LocalEventTime>),
    /// fire after choice value with these weight as random.
    WeightedIndex(Vec<(LocalEventTime, u8)>),
}

impl EventTimer {
    /// calculate time for event timer as local time
    fn to_local_time<R: Rng + ?Sized>(
        &self,
        rng: &mut R,
    ) -> Result<LocalEventTime, ScheduleEventError> {
        match &self {
            EventTimer::Time(timeout) => Ok(*timeout),
            EventTimer::Uniform(range) => Ok(Uniform::from(range.clone()).sample(rng)),
            EventTimer::WeightedIndex(items) => {
                let dist = WeightedIndex::new(items.iter().map(|item| item.1))?;
                Ok(items
                    // always success because sampler is constructed from list of the (LocalEventTimer, weight)s.
                    .get(dist.sample(rng))
                    .unwrap()
                    .0)
            }
        }
    }
}

/// event schedule
#[derive(Debug, Clone)]
pub enum Schedule {
    /// fire at immediate timing
    Immediate,
    /// fire after specify time
    Timeout(EventTimer),
    /// fire everytime
    Everytime,
    /// fire every specify time
    EveryInterval(EventTimer),
    /// fire every specify time only specify count
    Repeat(u8, EventTimer),
}

impl Schedule {
    /// calculate time for fire timing
    fn to_local_timer<R: Rng + ?Sized>(
        &self,
        rng: &mut R,
    ) -> Result<LocalEventTime, ScheduleEventError> {
        match &self {
            Schedule::Immediate => Ok(1),
            Schedule::Timeout(timeout) => timeout.to_local_time(rng),
            Schedule::Everytime => Ok(1),
            Schedule::EveryInterval(interval) => interval.to_local_time(rng),
            Schedule::Repeat(count, interval) => {
                if *count == 0 {
                    return Err(ScheduleEventError::CannotFireEvent);
                }

                return interval.to_local_time(rng);
            }
        }
    }

    /// convert to next schedule
    /// if cannot calc next schedule time then return None else return Some(schedule).
    fn to_next(&self) -> Option<Schedule> {
        match &self {
            Schedule::Immediate
            | Schedule::Timeout(_)
            | Schedule::Repeat(0, _)
            | Schedule::Repeat(1, _) => None,
            Schedule::Everytime => Some(Schedule::Everytime),
            Schedule::EveryInterval(interval) => Some(Schedule::EveryInterval(interval.clone())),
            Schedule::Repeat(count, interval) => {
                Some(Schedule::Repeat(count - 1, interval.clone()))
            }
        }
    }
}

/// 0 is the highest priority, u8::Max is the lowest priority.
pub type Priority = u8;

/// event scheduler
#[derive(Debug, Clone)]
pub struct EventScheduler<E: Event> {
    /// event list with inserted order by LocalEventTime's asc.
    event_list: Vec<(LocalEventTime, Schedule, Priority, E)>,
}

impl<E: Event> EventScheduler<E> {
    /// initializer
    pub(crate) fn new() -> Self {
        EventScheduler { event_list: vec![] }
    }

    /// calc next state and fetch fired events
    pub(crate) fn next_time_and_fire<R: Rng + ?Sized>(
        &mut self,
        rng: &mut R,
    ) -> Vec<(Priority, E)> {
        let mut removed: usize = 0;
        for event in self.event_list.iter_mut() {
            if event.0 > 0 {
                event.0 -= 1;
            }
            if event.0 == 0 {
                removed += 1;
            }
        }

        let fired_events: Vec<(Schedule, Priority, E)> = self
            .event_list
            .drain(0..removed)
            .map(|(_, s, pty, e)| (s, pty, e))
            .collect();

        // reschedule for calculated next event schedule
        for (schedule, pty, event) in fired_events.iter() {
            if let Some(next_schedule) = schedule.to_next() {
                // scheduled event's schedule is already validated
                self.schedule(rng, next_schedule, *pty, event.clone())
                    .unwrap();
            }
        }

        return fired_events.into_iter().map(|(_, p, e)| (p, e)).collect();
    }

    //
    // get state of scheduler state
    //

    /// judge exist scheduled event
    pub fn have_event(&self) -> bool {
        !self.event_list.is_empty()
    }

    /// get length of scheduled events
    pub fn count(&self) -> usize {
        self.event_list.len()
    }

    //
    // schedule event
    //

    /// clear all scheduled events
    pub fn clear(&mut self) {
        self.event_list.clear();
    }

    /// remove scheduled events when predicate function is true
    pub fn remove_when<P>(&mut self, mut predicate: P)
    where
        P: FnMut(&(LocalEventTime, Schedule, Priority, E)) -> bool,
    {
        self.event_list.retain(|state| !predicate(state))
    }

    /// retains only the scheduled events specified by the predicate.
    #[allow(unused_mut)]
    pub fn retain<P>(&mut self, mut predicate: P)
    where
        P: FnMut(&(LocalEventTime, Schedule, Priority, E)) -> bool,
    {
        self.event_list.retain(predicate)
    }

    /// store event with scheduling
    pub fn schedule<R: Rng + ?Sized>(
        &mut self,
        rng: &mut R,
        schedule: Schedule,
        priority: Priority,
        event: E,
    ) -> Result<(), ScheduleEventError> {
        let mut index: usize = 0;
        let timer: LocalEventTime = schedule.to_local_timer(rng)?;

        for (count, _, sch_priority, _) in self.event_list.iter() {
            if (&timer == count && &priority < sch_priority) || &timer < count {
                break;
            }
            index += 1;
        }
        self.event_list
            .insert(index, (timer, schedule, priority, event));
        Ok(())
    }

    /// store event with scheduling when user judge ok from all scheduled events
    pub fn schedule_when<R: Rng + ?Sized, P>(
        &mut self,
        rng: &mut R,
        schedule: Schedule,
        priority: Priority,
        event: E,
        predicate: P,
    ) -> Result<(), ScheduleEventError>
    where
        P: FnOnce(&Self) -> bool,
    {
        if !predicate(&self) {
            return Ok(());
        }
        self.schedule(rng, schedule, priority, event)
    }

    /// store event which fire at immediate timing
    pub fn immediate<R: Rng + ?Sized>(
        &mut self,
        rng: &mut R,
        priority: Priority,
        event: E,
    ) -> Result<(), ScheduleEventError> {
        self.schedule(rng, Schedule::Immediate, priority, event)
    }

    /// store event which fire after timeout
    pub fn timeout<R: Rng + ?Sized>(
        &mut self,
        rng: &mut R,
        timeout: EventTimer,
        priority: Priority,
        event: E,
    ) -> Result<(), ScheduleEventError> {
        self.schedule(rng, Schedule::Timeout(timeout), priority, event)
    }

    /// store event which fire every time
    pub fn everytime<R: Rng + ?Sized>(
        &mut self,
        rng: &mut R,
        priority: Priority,
        event: E,
    ) -> Result<(), ScheduleEventError> {
        self.schedule(rng, Schedule::Everytime, priority, event)
    }

    /// store event which fire every interval
    pub fn every_interval<R: Rng + ?Sized>(
        &mut self,
        rng: &mut R,
        interval: EventTimer,
        priority: Priority,
        event: E,
    ) -> Result<(), ScheduleEventError> {
        self.schedule(rng, Schedule::EveryInterval(interval), priority, event)
    }

    /// store event which fire every interval only count
    pub fn repeat<R: Rng + ?Sized>(
        &mut self,
        rng: &mut R,
        count: u8,
        interval: EventTimer,
        priority: Priority,
        event: E,
    ) -> Result<(), ScheduleEventError> {
        self.schedule(rng, Schedule::Repeat(count, interval), priority, event)
    }
}
