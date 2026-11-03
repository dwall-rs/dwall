//! AC/battery state for the light/dark switch.

use windows::Win32::System::Power::{GetSystemPowerStatus, SYSTEM_POWER_STATUS};

/// Whether an `ACLineStatus` value means the machine is drawing from its battery.
///
/// 0 is offline (battery), 1 is online (AC), 255 is unknown. Unknown must not force
/// dark: a desktop without a battery reports 255, and forcing dark there would make
/// the setting look broken.
fn battery_from_ac_status(ac_line_status: u8) -> bool {
    ac_line_status == 0
}

/// Whether the machine is currently running on battery power.
pub(crate) fn is_on_battery() -> bool {
    let mut status = SYSTEM_POWER_STATUS::default();

    if unsafe { GetSystemPowerStatus(&mut status) }.is_err() {
        return false;
    }

    battery_from_ac_status(status.ACLineStatus)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_offline_means_battery() {
        assert!(battery_from_ac_status(0));
        assert!(!battery_from_ac_status(1));
        assert!(
            !battery_from_ac_status(255),
            "an unknown state must not force dark"
        );
    }
}
