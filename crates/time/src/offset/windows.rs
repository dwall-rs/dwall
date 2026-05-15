use windows::Win32::System::Time::{GetTimeZoneInformation, TIME_ZONE_INFORMATION};

/// refer: <https://learn.microsoft.com/en-us/windows/win32/api/timezoneapi/nf-timezoneapi-gettimezoneinformation#return-value>
const TIME_ZONE_ID_DAYLIGHT: u32 = 2;

pub fn system_utc_offset_secs() -> i32 {
    unsafe {
        let mut tz: TIME_ZONE_INFORMATION = std::mem::zeroed();
        let tz_id = GetTimeZoneInformation(&mut tz);

        let dst_bias = match tz_id {
            TIME_ZONE_ID_DAYLIGHT => tz.DaylightBias,
            _ => tz.StandardBias,
        };

        -((tz.Bias + dst_bias) * 60)
    }
}
