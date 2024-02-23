use std::future::Future;
use std::sync::Arc;
use std::time::Duration;
use zbus::{CacheProperties, dbus_proxy};

use zbus::{AsyncDrop, Message, Interface, ProxyDefault, PropertyStream, zvariant, SignalStream};
use futures_util::{TryStream, TryStreamExt, StreamExt, SinkExt};
use serde::{Deserialize, Serialize};
use crate::State;
use crate::upower::{UPowerProxy, DeviceProxy};


#[derive(Debug, Serialize, Deserialize)]
struct BatteryIconsConfig {
    charging: String,
    discharging: Vec<String>
}

impl Default for BatteryIconsConfig {
    fn default() -> Self {
        return Self {
            charging: " C".into(),
            discharging: vec![
                "00".into(),
                "10".into(),
                "20".into(),
                "30".into(),
                "40".into(),
                "50".into(),
                "60".into(),
                "70".into(),
                "80".into(),
                "90".into(),
            ]
        }
    }
}


#[derive(Debug, Serialize, Deserialize)]
pub struct UpowerConfig {
    icons: BatteryIconsConfig
}

impl Default for UpowerConfig {
    fn default() -> Self {
        Self {
            icons: BatteryIconsConfig::default()
        }
    }
}

struct BatteryMonitor<'c> {
    state: State,
    upower: UPowerProxy<'c>,
    device: DeviceProxy<'c>
}

#[derive(Debug, Serialize, Deserialize)]
struct BatteryUpdate<'c> {
    capacity: f64,
    time_to_empty: i64,
    time_to_full: i64,
    on_power: bool,
    icon: &'c str
}


impl<'c> BatteryMonitor<'c> {

    async fn new(state: State) -> anyhow::Result<Self> {

        let upower= UPowerProxy::builder(state.system()).build().await?;

        let path = upower.get_display_device().await?;
        println!("Path: {}", path);
        let device = DeviceProxy::builder(state.system())
            .cache_properties(CacheProperties::Yes)
            .destination("org.freedesktop.UPower")?
            .path("/org/freedesktop/UPower/devices/battery_BAT0")?
            .build().await?;

        let mon = Self {
            state,
            upower,
            device
        };

        let config = &mon.state.config().upower;

        let capacity = mon.device.percentage().await.unwrap_or(0.0);
        let on_power = !mon.upower.on_battery().await.unwrap_or(true);
        let icon = match (on_power, capacity) {
            (true, _) => &config.icons.charging,
            (false, p) if p >= 90.0 => &config.icons.discharging[9],
            (false, p) if p >= 80.0 => &config.icons.discharging[8],
            (false, p) if p >= 70.0 => &config.icons.discharging[7],
            (false, p) if p >= 60.0 => &config.icons.discharging[6],
            (false, p) if p >= 50.0 => &config.icons.discharging[5],
            (false, p) if p >= 40.0 => &config.icons.discharging[4],
            (false, p) if p >= 30.0 => &config.icons.discharging[3],
            (false, p) if p >= 20.0 => &config.icons.discharging[2],
            (false, p) if p >= 10.0 => &config.icons.discharging[1],
            (false, _) => &config.icons.discharging[0]
        };

        let status = BatteryUpdate {
            capacity,
            time_to_full: mon.device.time_to_full().await.unwrap_or(0),
            time_to_empty: mon.device.time_to_empty().await.unwrap_or(0),
            on_power,
            icon
        };


        if let Err(e) = mon.state.send_update("battery_update", &status).await {
            println!("Error updating: {}", e);
        }



        Ok(mon)
    }

    async fn upower_stream(&self) -> zbus::Result<SignalStream<'c>> {
        self.upower.receive_all_signals().await
    }

    async fn device_state(&self) -> SignalStream<'c> {
        self.device.receive_all_signals().await.unwrap()
    }


    async fn update(&self) {

        let capacity = self.device.cached_percentage().unwrap_or(Some(0.0)).unwrap_or(0.0);
        let on_power = !self.upower.cached_on_battery().unwrap_or(Some(true)).unwrap_or(true);

        dbg!(capacity);
        dbg!(on_power);

        let config = &self.state.config().upower;
        let icon = match (on_power, capacity) {
            (true, _) => &config.icons.charging,
            (false, p) if p >= 90.0 => &config.icons.discharging[9],
            (false, p) if p >= 80.0 => &config.icons.discharging[8],
            (false, p) if p >= 70.0 => &config.icons.discharging[7],
            (false, p) if p >= 60.0 => &config.icons.discharging[6],
            (false, p) if p >= 50.0 => &config.icons.discharging[5],
            (false, p) if p >= 40.0 => &config.icons.discharging[4],
            (false, p) if p >= 30.0 => &config.icons.discharging[3],
            (false, p) if p >= 20.0 => &config.icons.discharging[2],
            (false, p) if p >= 10.0 => &config.icons.discharging[1],
            (false, _) => &config.icons.discharging[0],
            _ => return
        };

        let status = BatteryUpdate {
            capacity,
            time_to_full: self.device.cached_time_to_full().unwrap_or(Some(0)).unwrap_or(0),
            time_to_empty: self.device.cached_time_to_empty().unwrap_or(Some(0)).unwrap_or(0),
            on_power,
            icon
        };


        if let Err(e) = self.state.send_update("battery_update", &status).await {
            println!("Error updating: {}", e);
        }


    }

}






pub async fn monitor_battery(state: State) -> anyhow::Result<()> {


    // let properties = zbus::fdo::PropertiesProxy::builder(state.system())
    //     .destination("org.freedesktop.UPower")?
    //     .path("/org/freedesktop/UPower/devices/DisplayDevice")?
    //     .build().await?;
    //
    // let mut stream = properties.receive_properties_changed().await?;
    //
    // let mut device_data = Box::new(DeviceProperties::new(state.system()).await?);
    //
    // let mut serial = 0;

    let manager = match BatteryMonitor::new(state.clone()).await {
        Ok(m) => m,
        Err(e) => {
            println!("Err {}", &e);
            return Err(e.into());
        }
    };
    let heartbeat_duration = Duration::from_secs(20);

    let mut upower_stream = manager.upower_stream().await?;
    let mut state_stream = manager.device_state().await;

    loop {

        tokio::select!(
            Some(msg) = upower_stream.next() => {
                println!("Upower: {}", msg);
            }
            Some(msg)= state_stream.next() => {
                // println!("Upower state {:?}", msg.get().await.unwrap());
                manager.update().await;
            }
            _ = tokio::time::sleep(heartbeat_duration) => {
                println!("power Heartbeat");
                manager.update().await;
            }
            );

        // if let Some(change) = stream.next().await {
        //     serial += 1;
        //     let body = change.body::<zvariant::Structure>()?;
        //
        //     let fields = body.fields();
        //     if let Some(updates) = fields[1].downcast_ref::<zvariant::Dict>() {
        //         if let Err(ref e) = device_data.update(updates).await {
        //             println!("An error occurred: {}", e);
        //         }
        //
        //
        //     }
        //     if let Err(e) = state.send_update("upower_status", &*device_data).await {
        //         println!("An error occurred: {}", e);
        //     }
        // }

    }



    // stream.async_drop().await;


    Ok(())
}