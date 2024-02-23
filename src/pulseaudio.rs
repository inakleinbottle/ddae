use crate::State;
use pipewire::MainLoop;
use std::cell::RefCell;
use std::future::Future;
use std::pin::Pin;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::Arc;
use std::task::Poll;
use std::time::Duration;
use tokio;

use pulse::context::subscribe::{Facility, InterestMaskSet, Operation};
use pulse::context::{self, Context};
use pulse::mainloop::api::Mainloop as MainLoopAPI;
use pulse::mainloop::standard::Mainloop;

fn subscribe_callback(status: bool) {
    println!("success");
}

enum PAMessage {}

fn monitor_pa_worker(state: State, mut tx: Sender<PAMessage>, rx: Receiver<PAMessage>) {
    let mut mainloop = Mainloop::new().unwrap();
    let mut context = Context::new(&mainloop, "ddae").expect("failed to create context");

    context
        .connect(None, pulse::context::FlagSet::all(), None)
        .expect("connection failed");

    while !context.get_state().is_good() {
        mainloop.iterate(false);
    }

    let sub_cb = context.subscribe(context::subscribe::InterestMaskSet::ALL, |state| {
        assert!(state);
        println!("subscribed to all events");
    });

    context.set_subscribe_callback(Some(Box::new(move |fac, op, index| {
        println!("Event");
        if let Some(facility) = fac {
            println!(" facility: {:?}", facility);
        }
        if let Some(operation) = op {
            println!(" operation: {:?}", operation);
        }
    })));
}

pub async fn monitor_pulseaudio(state: State) -> anyhow::Result<()> {
    let (mut tx, mut rx) = mpsc::channel();

    tokio::task::spawn_blocking(move || monitor_pa_worker(state, tx, rx));

    Ok(())
}
