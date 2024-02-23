mod bluetooth;
mod device;
mod wired;
mod wireless;

pub use bluetooth::BluetoothProxy;
pub use device::{DeviceProxy};
pub use wired::WiredProxy;
pub use wireless::{WirelessProxy};


