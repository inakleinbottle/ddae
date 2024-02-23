
use std::sync::Arc;
use std::time::Duration;

use zbus::{AsyncDrop, Interface, Proxy, zvariant};
use futures_util::{TryStream, StreamExt};
use serde::{Serialize, Deserialize};
use tokio;

use anyhow;
use zbus::fdo::PropertiesChangedStream;
use crate::State;

use tokio::sync::Mutex;
use zbus::zvariant::OwnedObjectPath;
use crate::network_manager::{BluetoothProxy, ActiveConnectionProxy, DeviceProxy, NetworkManagerProxy, WiredProxy, WirelessProxy};


struct NetworkManager<'c> {
    state: State,
    manager: NetworkManagerProxy<'c>,
}

impl<'c> NetworkManager<'c> {

    pub async fn new(state: State) -> anyhow::Result<NetworkManager<'c>> {

        let prop_proxy = match zbus::fdo::PropertiesProxy::builder(state.system())
            .destination("org.freedesktop.NetworkManager")?
            .path("/org/freedesktop/NetworkManager")?
            .build().await {
            Ok(p) => p,
            Err(e) => {
                println!("Error: {}", &e);
                return Err(e.into());
            }
        };

        println!("Get manager");
        let manager = NetworkManagerProxy::new(state.system()).await?;

        println!("Acquired manager");
        println!("Wifi online: {}", manager.wireless_enabled().await?);


        let active = manager.active_connections().await?;
        for oop in &active {
            println!("Path {}", oop);
            let dev = ActiveConnectionProxy::builder(state.system())
                .destination("org.freedesktop.NetworkManager")?
                .path(oop)?
                .build().await.unwrap();
            println!("Get state");
            println!("{}", dev.id().await.unwrap());
        }


        let mut result = NetworkManager {
            state,
            manager,
        };

        Ok(result)
    }


    async fn update(&mut self, stream: &mut PropertiesChangedStream<'_>) {
    }
}


pub async fn monitor_network(state: State) -> anyhow::Result<()> {

    // let mut manager = NetworkManager::new(state);

    let mut network = NetworkManager::new(state.clone()).await?;

    let mut nw_enabled_stream = network.manager.receive_networking_enabled_changed().await;
    let mut primary = network.manager.primary_connection().await.ok();
    let mut primary_changed_stream = network.manager.receive_primary_connection_changed().await;

    loop {
        println!("Network-loop");

        tokio::select!(
            Some(enabled) = nw_enabled_stream.next() => {
                match enabled.get().await {
                    Ok(true) => println!("Network enabled"),
                    Ok(false) => println!("netork disabled"),
                    _ => println!("Something wrong")
                }
            }
            new_primary = primary_changed_stream.next() => {
                if let Some(ref p) = new_primary {
                    // primary = p;

                    println!("Updating primary interface: ");
                } else {
                    println!("Primary interface disabled");
                }

            },
            _ = tokio::time::sleep(Duration::from_secs(20)) => {

                println!("Network heartbeat");
            }
        )


        // if let Err(e) = manager.update().await {
        //     println!("An error occurred: {}", e);
        // }

    }



}