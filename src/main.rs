use std::time::Duration;
use tokio;
use anyhow::Result;
use zbus::{Connection, MessageStream, MatchRule, Message};
use futures_util::{TryStreamExt, StreamExt, pin_mut};

use ddae::{State, monitor_battery, monitor_network, monitor_pulseaudio};

#[tokio::main]
async fn main() -> Result<()> {

    let bus = Connection::system().await?;

    let state = State::new().await?;

    let bat_mon_handle = tokio::spawn(monitor_battery(state.clone()));
    tokio::pin!(bat_mon_handle);
    let net_mon_handle = tokio::spawn(monitor_network(state.clone()));
    tokio::pin!(net_mon_handle);
    // let pw_mon_handle = tokio::spawn(monitor_pipewire(state.clone()));
    // tokio::pin!(pw_mon_handle);
    let pa_mon_handle = tokio::spawn(monitor_pulseaudio(state.clone()));
    tokio::pin!(pa_mon_handle);

    loop {

        tokio::select!(
            // Err(e) = &mut pw_mon_handle => { println!("pw monitor failed: {}", e); break; }
            Err(e) = &mut pa_mon_handle => { println!("PA monitor failed, {}", e); break; }
            Err(e) = &mut bat_mon_handle => { println!("battery monitor failed: {}", e); break }
            Err(e) = &mut net_mon_handle => { println!("network monitor failed: {}", e); break }
        );





    }

    // bat_mon_handle.abort();
    // net_mon_handle.abort();

    Ok(())
}
