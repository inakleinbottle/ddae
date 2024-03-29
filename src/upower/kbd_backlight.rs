//! # DBus interface proxy for: `org.freedesktop.UPower.KbdBacklight`
//!
//! This code was generated by `zbus-xmlgen` `3.1.1` from DBus introspection data.
//! Source: `org.freedesktop.UPower.KbdBacklight.xml`.
//!
//! You may prefer to adapt it, instead of using it verbatim.
//!
//! More information can be found in the
//! [Writing a client proxy](https://dbus.pages.freedesktop.org/zbus/client.html)
//! section of the zbus documentation.
//!

use zbus::dbus_proxy;

#[dbus_proxy(
    interface = "org.freedesktop.UPower.KbdBacklight",
    assume_defaults = true
)]
trait KbdBacklight {
    /// GetBrightness method
    fn get_brightness(&self) -> zbus::Result<i32>;

    /// GetMaxBrightness method
    fn get_max_brightness(&self) -> zbus::Result<i32>;

    /// SetBrightness method
    fn set_brightness(&self, value: i32) -> zbus::Result<()>;

    /// BrightnessChanged signal
    #[dbus_proxy(signal)]
    fn brightness_changed(&self, value: i32) -> zbus::Result<()>;

    /// BrightnessChangedWithSource signal
    #[dbus_proxy(signal)]
    fn brightness_changed_with_source(&self, value: i32, source: &str) -> zbus::Result<()>;
}
