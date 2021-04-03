//! SimRs is discrete time simulator with event which fire at scheduled timing.

use crate::event::{Event, EventScheduler};
use crate::model::Model;
use rand::Rng;
use std::mem;

pub mod event;
pub mod model;

/// Timer for user
pub type GlobalEventTime = u64;

/// simulator
#[derive(Debug, Clone)]
pub struct SimRs<M, E, Rec>
where
    M: Model<Rec, ModelEvent = E>,
    E: Event,
{
    model: M,
    recorder: Rec,
    scheduler: EventScheduler<E>,
}

impl<M, E, Rec> Default for SimRs<M, E, Rec>
where
    M: Model<Rec, ModelEvent = E> + Default,
    E: Event,
    Rec: Default,
{
    fn default() -> Self {
        Self {
            model: Default::default(),
            recorder: Default::default(),
            scheduler: EventScheduler::new(),
        }
    }
}

impl<M, E, Rec> SimRs<M, E, Rec>
where
    M: Model<Rec, ModelEvent = E>,
    E: Event,
{
    /// create simulator from model
    pub fn create_from(model: M, recorder: Rec) -> Self {
        Self {
            model,
            recorder,
            scheduler: EventScheduler::new(),
        }
    }

    /// getter for model
    pub fn get_model(&self) -> &M {
        &self.model
    }

    /// getter for scheduler
    pub fn get_scheduler(&self) -> &EventScheduler<E> {
        &self.scheduler
    }

    /// getter for recorder
    pub fn get_recorder(&self) -> &Rec {
        &self.recorder
    }

    /// getter for recorder
    pub fn get_recorder_as_mut(&mut self) -> &mut Rec {
        &mut self.recorder
    }

    /// swap new and old recorder with get old recorder.
    pub fn swap_recorder(&mut self, new_recorder: Rec) -> Rec {
        mem::replace(&mut self.recorder, new_recorder)
    }

    // -----------
    // methods for run simulation
    // -----------

    /// initialize simulator
    pub fn initialize<R: Rng + ?Sized>(&mut self, rng: &mut R) -> &mut Self {
        self.scheduler.clear();
        self.model.initialize(rng, &mut self.recorder);
        self.model
            .at_first_frame(rng, &mut self.recorder, &mut self.scheduler);
        self
    }

    /// run simulate for frames
    pub fn run<R: Rng + ?Sized>(&mut self, rng: &mut R, frame_count: GlobalEventTime) {
        for _ in 0..frame_count {
            self.step(rng);
        }
    }

    /// run simulate for one frame
    pub fn step<R: Rng + ?Sized>(&mut self, rng: &mut R) {
        let fired: Vec<E> = self.scheduler.next_time_and_fire(rng);
        self.model
            .step(rng, &mut self.recorder, &mut self.scheduler, fired);
    }

    /// run simulation until condition is true
    pub fn run_until<R: Rng + ?Sized, F>(&mut self, rng: &mut R, judge: F)
    where
        F: Fn(&mut R, &M, &EventScheduler<E>) -> bool,
    {
        while judge(rng, &self.model, &self.scheduler) {
            self.step(rng);
        }
    }
}
