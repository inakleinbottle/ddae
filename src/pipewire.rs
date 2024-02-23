use std::collections::LinkedList;
use std::sync::{Arc, RwLock};
use std::time::Duration;
use std::ops::Deref;



use tokio::sync::mpsc;
use pipewire::{MainLoop, Context, node::Node};
use pipewire::types::ObjectType;

use crate::State;

#[derive(Clone)]
struct Nodes(Arc<RwLock<LinkedList<Node>>>);

impl Nodes {
    fn new() -> Nodes {
        Nodes(Arc::new(RwLock::new(LinkedList::new())))
    }
}

impl Deref for Nodes {
    type Target = RwLock<LinkedList<Node>>;

    fn deref(&self) -> &Self::Target {
        self.0.deref()
    }
}

unsafe impl Send for Nodes {}
unsafe impl Sync for Nodes {}


pub async fn monitor_pipewire(state: State) -> anyhow::Result<()> {
    let (rx, mut tx) = mpsc::channel::<String>(5);


    let nodes = Nodes::new();
    let node_list = nodes.clone();

    let handle = tokio::task::spawn_blocking(move || -> anyhow::Result<()> {
        pipewire::init();

        let main_loop = MainLoop::new()?;
        let context = Context::new(&main_loop)?;
        let core = context.connect(None)?;
        let registry = core.get_registry()?;


        let nlist = node_list.clone();

        let _listener = registry.add_listener_local()
            .global(move |global| {

                match global.type_ {
                    ObjectType::Node => {
                        println!("Node: {:?}", global);
                    }
                    _ => {}
                }


            })
            .register();



        let props = pipewire::Properties::new();
        let md = core.create_object::<pipewire::metadata::Metadata, _>("metadata", &props)?;
        let _md_listener = md.add_listener_local()
            .property(|subject, key, type_, value| {
                match (key, type_, value) {
                    (Some(k), Some(t), Some(v)) => println!("{}: {} type: {}, value: {}", subject, k, t, v),
                    _ => println!("{}", subject)
                }
                0
            })
            .register();

        let _core_listener = core.add_listener_local()
            .info(|info| println!("PWInfo {:?}", info))
            .done(|i, aseq| println!("done {}: {:?}", i, aseq))
            .error(|a, b, c, info| println!("Error {}", info))
            .register();

        main_loop.run();



        Ok(())
    });

    loop {
        tokio::time::sleep(Duration::from_secs(30)).await;
    }


    Ok(())
}
