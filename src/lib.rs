mod battery;
mod network;
mod network_manager;

mod upower;
mod pipewire;
mod pulseaudio;

use std::sync::Arc;
use std::ops::{Deref, DerefMut};
use std::ffi::{OsString, OsStr};
use std::pin::Pin;

use tokio;
use serde::{Serialize, Deserialize};
use zbus::Connection;

pub use battery::monitor_battery;
pub use network::monitor_network;
pub use pipewire::monitor_pipewire;
pub use pulseaudio::monitor_pulseaudio;
use crate::battery::UpowerConfig;

#[derive(Default, Serialize, Deserialize)]
pub struct Config {
    upower: UpowerConfig
}




pub(crate) struct StateInner {
    pub(crate) config: Config,
    pub(crate) session_bus_conn: Connection,
    pub(crate) system_bus_conn: zbus::Connection
}


impl StateInner {

    async fn new() -> anyhow::Result<StateInner> {
        let inner = StateInner {
            config: Config::default(),
            system_bus_conn: Connection::system().await?,
            session_bus_conn: Connection::session().await?
        };

        // inner.system_bus_conn.request_name("com.inakleinbottle.ddae").await?;
        // inner.session_bus_conn.request_name("com.inakleinbottle.ddae").await?;

        Ok(inner)
    }
}

#[derive(Clone)]
pub struct State(Arc<StateInner>);


impl State {

    pub async fn new() -> anyhow::Result<State> {


        Ok(State(Arc::new(StateInner::new().await?)))
    }

    pub fn system(&self) -> &Connection {
        &self.0.system_bus_conn
    }

    pub fn session(&self) -> &Connection {
        &self.0.session_bus_conn
    }

    pub async fn send_update(&self, variable: &str, data: &impl Serialize) -> anyhow::Result<()> {

        let json = serde_json::to_string(data)?;
        println!("{}", &json);

        Ok(())
    }

    pub fn config(&self) -> &Config {
        &self.0.config
    }

}








#[derive(Debug, Serialize, Deserialize)]
pub enum ListenerType {
    FilePoll {
        interval: i32,
        file: String
    },
    CommandPoll {
        interval: i32,
        command: String
    },
    SocketRead {
        number: i32
    },
    UnixSocketRead {
        addr: String
    },
    DBusSignal {

    }
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Action {
    NoAction,
    RunCommand {
        command: String
    },
    SocketWrite {
        addr: String,
        content: String
    }
}



#[derive(Debug, Serialize, Deserialize)]
pub struct HandlerConfig {
    name: String,
    listener_type: ListenerType,
    action: Action
}





