mod error;
#[cfg(unix)]
mod unix;
#[cfg(windows)]
mod windows;

use std::fmt;

pub use self::error::ParseOffsetError;
#[cfg(unix)]
pub use self::unix::system_utc_offset_secs;
#[cfg(windows)]
pub use self::windows::system_utc_offset_secs;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Offset {
    /// UTC offset, in seconds. Range [-50400, 50400]
    pub(super) seconds: i32,
}

impl Offset {
    pub fn local_offset() -> Self {
        Self {
            seconds: system_utc_offset_secs(),
        }
    }

    pub const fn seconds(&self) -> i32 {
        self.seconds
    }
}

impl fmt::Display for Offset {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let total_sec = self.seconds;
        let sign = if total_sec < 0 { '-' } else { '+' };
        let abs_sec = total_sec.unsigned_abs(); // u32
        let hours = abs_sec / 3600;
        let minutes = abs_sec % 3600 / 60;
        let seconds = abs_sec % 60;

        if seconds == 0 {
            write!(f, "{sign}{hours:02}:{minutes:02}")
        } else {
            write!(f, "{sign}{hours:02}:{minutes:02}:{seconds:02}")
        }
    }
}

impl std::str::FromStr for Offset {
    type Err = ParseOffsetError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let b = s.as_bytes();

        // At least 3 bytes required: sign + two-digit hour, e.g., "+00"
        if b.len() < 3 {
            return Err(ParseOffsetError::InvalidFormat(s.to_string()));
        }

        let sign: i32 = match b[0] {
            b'+' => 1,
            b'-' => -1,
            _ => return Err(ParseOffsetError::InvalidFormat(s.to_string())),
        };

        // Parse two-digit hour
        let h1 = digit(b[1]).ok_or(ParseOffsetError::InvalidFormat(s.to_string()))?;
        let h2 = digit(b[2]).ok_or(ParseOffsetError::InvalidFormat(s.to_string()))?;
        let hours = (h1 * 10 + h2) as u32;

        // Must be followed by ':'
        if b.get(3) != Some(&b':') || b.len() < 6 {
            return Err(ParseOffsetError::InvalidFormat(s.to_string()));
        }

        // Parse two-digit minute
        let m1 = digit(b[4]).ok_or(ParseOffsetError::InvalidFormat(s.to_string()))?;
        let m2 = digit(b[5]).ok_or(ParseOffsetError::InvalidFormat(s.to_string()))?;
        let minutes = (m1 * 10 + m2) as u32;
        if minutes > 59 {
            return Err(ParseOffsetError::InvalidMinute(minutes));
        }

        let mut total_secs = hours * 3600 + minutes * 60;

        // Optional seconds part
        if b.len() == 9 {
            if b[6] != b':' {
                return Err(ParseOffsetError::InvalidFormat(s.to_string()));
            }
            let s1 = digit(b[7]).ok_or(ParseOffsetError::InvalidFormat(s.to_string()))?;
            let s2 = digit(b[8]).ok_or(ParseOffsetError::InvalidFormat(s.to_string()))?;
            let seconds = (s1 * 10 + s2) as u32;
            if seconds > 59 {
                return Err(ParseOffsetError::InvalidSecond(seconds));
            }
            total_secs += seconds;
        } else if b.len() != 6 {
            // Length is neither 6 (HH:MM) nor 9 (HH:MM:SS)
            return Err(ParseOffsetError::InvalidFormat(s.to_string()));
        }

        let final_seconds = sign * total_secs as i32;
        if !(-50400..=50400).contains(&final_seconds) {
            return Err(ParseOffsetError::OutOfRange(final_seconds));
        }

        Ok(Offset {
            seconds: final_seconds,
        })
    }
}

/// Convert a byte to a digit 0..=9
fn digit(b: u8) -> Option<u8> {
    if b.is_ascii_digit() {
        Some(b - b'0')
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::Offset;
    use super::ParseOffsetError;

    // =========================================================
    // 1. Roundtrip consistency from_seconds / seconds
    // =========================================================

    #[test]
    fn test_roundtrip_zero() {
        let o = Offset { seconds: 0 };
        assert_eq!(o.seconds(), 0);
    }

    #[test]
    fn test_roundtrip_positive_whole_hour() {
        // UTC+8 (Beijing)
        let o = Offset { seconds: 8 * 3600 };
        assert_eq!(o.seconds(), 28800);
    }

    #[test]
    fn test_roundtrip_negative_whole_hour() {
        // UTC-5 (New York EST)
        let o = Offset { seconds: -5 * 3600 };
        assert_eq!(o.seconds(), -18000);
    }

    #[test]
    fn test_roundtrip_half_hour() {
        // UTC+5:30 (India IST)
        let o = Offset {
            seconds: 5 * 3600 + 30 * 60,
        };
        assert_eq!(o.seconds(), 19800);
    }

    #[test]
    fn test_roundtrip_negative_half_hour() {
        // UTC-9:30 (Marquesas Islands)
        let o = Offset {
            seconds: -(9 * 3600 + 30 * 60),
        };
        assert_eq!(o.seconds(), -34200);
    }

    #[test]
    fn test_roundtrip_quarter_hour() {
        // UTC+5:45 (Nepal NPT)
        let o = Offset {
            seconds: 5 * 3600 + 45 * 60,
        };
        assert_eq!(o.seconds(), 20700);
    }

    #[test]
    fn test_roundtrip_max() {
        // UTC+14:00 (maximum legal offset)
        let o = Offset { seconds: 14 * 3600 };
        assert_eq!(o.seconds(), 50400);
    }

    #[test]
    fn test_roundtrip_min() {
        // UTC-12:00 (minimum legal offset)
        let o = Offset {
            seconds: -12 * 3600,
        };
        assert_eq!(o.seconds(), -43200);
    }

    #[test]
    fn test_roundtrip_with_seconds() {
        // Rare offset with seconds (historical LMT)
        let o = Offset {
            seconds: 3 * 3600 + 7 * 60 + 33,
        };
        assert_eq!(o.seconds(), 3 * 3600 + 7 * 60 + 33);
    }

    // =========================================================
    // 2. Display formatting
    // =========================================================

    #[test]
    fn test_display_utc() {
        assert_eq!(Offset { seconds: 0 }.to_string(), "+00:00");
    }

    #[test]
    fn test_display_positive_whole_hours() {
        assert_eq!(Offset { seconds: 9 * 3600 }.to_string(), "+09:00"); // Tokyo
    }

    #[test]
    fn test_display_negative_whole_hours() {
        assert_eq!(Offset { seconds: -5 * 3600 }.to_string(), "-05:00"); // New York EST
    }

    #[test]
    fn test_display_positive_half_hour() {
        assert_eq!(
            Offset {
                seconds: 5 * 3600 + 30 * 60
            }
            .to_string(),
            "+05:30"
        ); // India IST
    }

    #[test]
    fn test_display_negative_half_hour() {
        assert_eq!(
            Offset {
                seconds: -(9 * 3600 + 30 * 60)
            }
            .to_string(),
            "-09:30"
        );
    }

    #[test]
    fn test_display_quarter_hour() {
        assert_eq!(
            Offset {
                seconds: 5 * 3600 + 45 * 60
            }
            .to_string(),
            "+05:45"
        ); // Nepal NPT
    }

    #[test]
    fn test_display_max_offset() {
        assert_eq!(Offset { seconds: 14 * 3600 }.to_string(), "+14:00");
    }

    #[test]
    fn test_display_min_offset() {
        assert_eq!(
            Offset {
                seconds: -12 * 3600
            }
            .to_string(),
            "-12:00"
        );
    }

    #[test]
    fn test_display_with_seconds_positive() {
        // +00:09:21 — historical local mean time offset
        let o = Offset {
            seconds: 9 * 60 + 21,
        };
        assert_eq!(o.to_string(), "+00:09:21");
    }

    #[test]
    fn test_display_with_seconds_negative() {
        let o = Offset {
            seconds: -(5 * 3600 + 30 * 60 + 15),
        };
        assert_eq!(o.to_string(), "-05:30:15");
    }

    #[test]
    fn test_display_seconds_zero_omitted() {
        // Ensure seconds part ":00" is not output for whole minutes
        let s = Offset { seconds: 3600 }.to_string();
        assert!(
            !s.ends_with(":00:00"),
            "Should not output extra :00 seconds"
        );
        assert_eq!(s, "+01:00");
    }

    #[test]
    fn test_display_leading_zeros() {
        // Hours and minutes must be zero-padded
        assert_eq!(Offset { seconds: 30 * 60 }.to_string(), "+00:30");
    }

    // =========================================================
    // 3. Sign boundaries
    // =========================================================

    #[test]
    fn test_sign_positive_one_second() {
        assert_eq!(Offset { seconds: 1 }.to_string(), "+00:00:01");
    }

    #[test]
    fn test_sign_negative_one_second() {
        assert_eq!(Offset { seconds: -1 }.to_string(), "-00:00:01");
    }

    #[test]
    fn test_sign_exactly_zero_is_positive() {
        // UTC displays as +00:00
        assert!(Offset { seconds: 0 }.to_string().starts_with('+'));
    }

    // =========================================================
    // 4. Real-world timezone snapshots
    // =========================================================

    #[test]
    fn test_real_world_kolkata() {
        // Asia/Kolkata  UTC+5:30
        let o = Offset { seconds: 19800 };
        assert_eq!(o.to_string(), "+05:30");
    }

    #[test]
    fn test_real_world_kathmandu() {
        // Asia/Kathmandu  UTC+5:45
        let o = Offset { seconds: 20700 };
        assert_eq!(o.to_string(), "+05:45");
    }

    #[test]
    fn test_real_world_marquesas() {
        // Pacific/Marquesas  UTC-9:30
        let o = Offset { seconds: -34200 };
        assert_eq!(o.to_string(), "-09:30");
    }

    #[test]
    fn test_real_world_lord_howe() {
        // Australia/Lord_Howe  daylight saving UTC+11:00, standard UTC+10:30
        let o = Offset {
            seconds: 10 * 3600 + 30 * 60,
        };
        assert_eq!(o.to_string(), "+10:30");
    }

    #[test]
    fn test_real_world_newfoundland() {
        // America/St_Johns  UTC-3:30
        let o = Offset {
            seconds: -(3 * 3600 + 30 * 60),
        };
        assert_eq!(o.to_string(), "-03:30");
    }

    // =========================================================
    // 5. Field derivation / trait implementations
    // =========================================================

    #[test]
    fn test_copy_semantics() {
        // If Offset is Copy, the original remains usable after assignment
        let a = Offset { seconds: 3600 };
        let b = a; // would fail to compile if not Copy
        assert_eq!(a.seconds(), b.seconds());
    }

    // =========================================================
    // FromStr tests
    // =========================================================

    #[test]
    fn test_parse_utc() {
        let o: Offset = "+00:00".parse().unwrap();
        assert_eq!(o.seconds(), 0);
        // also accept negative zero
        let o: Offset = "-00:00".parse().unwrap();
        assert_eq!(o.seconds(), 0);
    }

    #[test]
    fn test_parse_positive_whole_hour() {
        let o: Offset = "+08:00".parse().unwrap();
        assert_eq!(o.seconds(), 8 * 3600);
    }

    #[test]
    fn test_parse_negative_whole_hour() {
        let o: Offset = "-05:00".parse().unwrap();
        assert_eq!(o.seconds(), -5 * 3600);
    }

    #[test]
    fn test_parse_positive_half_hour() {
        let o: Offset = "+05:30".parse().unwrap();
        assert_eq!(o.seconds(), 19800);
    }

    #[test]
    fn test_parse_negative_half_hour() {
        let o: Offset = "-09:30".parse().unwrap();
        assert_eq!(o.seconds(), -34200);
    }

    #[test]
    fn test_parse_quarter_hour() {
        let o: Offset = "+05:45".parse().unwrap();
        assert_eq!(o.seconds(), 20700);
    }

    #[test]
    fn test_parse_with_seconds() {
        let o: Offset = "+03:07:33".parse().unwrap();
        assert_eq!(o.seconds(), 3 * 3600 + 7 * 60 + 33);
    }

    #[test]
    fn test_parse_negative_with_seconds() {
        let o: Offset = "-05:30:15".parse().unwrap();
        assert_eq!(o.seconds(), -(5 * 3600 + 30 * 60 + 15));
    }

    #[test]
    fn test_parse_max_offset() {
        let o: Offset = "+14:00".parse().unwrap();
        assert_eq!(o.seconds(), 50400);
    }

    #[test]
    fn test_parse_min_offset() {
        let o: Offset = "-14:00".parse().unwrap();
        assert_eq!(o.seconds(), -50400);
    }

    #[test]
    fn test_parse_leading_zeros() {
        let o: Offset = "+00:30".parse().unwrap();
        assert_eq!(o.seconds(), 1800);
    }

    // === Invalid inputs ===

    #[test]
    fn test_parse_missing_sign() {
        "00:30".parse::<Offset>().unwrap_err();
    }

    #[test]
    fn test_parse_wrong_sign() {
        "*05:00".parse::<Offset>().unwrap_err();
    }

    #[test]
    fn test_parse_missing_colon() {
        "+0800".parse::<Offset>().unwrap_err();
    }

    #[test]
    fn test_parse_too_short() {
        "+0".parse::<Offset>().unwrap_err();
    }

    #[test]
    fn test_parse_too_long() {
        "+05:00:00:00".parse::<Offset>().unwrap_err();
    }

    #[test]
    fn test_parse_non_digit_hour() {
        "+0A:00".parse::<Offset>().unwrap_err();
    }

    #[test]
    fn test_parse_minutes_out_of_range() {
        assert_eq!(
            "+05:60".parse::<Offset>().unwrap_err().to_string(),
            ParseOffsetError::InvalidMinute(60).to_string()
        );
    }

    #[test]
    fn test_parse_seconds_out_of_range() {
        assert_eq!(
            "+05:30:60".parse::<Offset>().unwrap_err().to_string(),
            ParseOffsetError::InvalidSecond(60).to_string()
        );
    }

    #[test]
    fn test_parse_offset_out_of_range_positive() {
        assert_eq!(
            "+14:01".parse::<Offset>().unwrap_err().to_string(),
            ParseOffsetError::OutOfRange(50460).to_string()
        );
    }

    #[test]
    fn test_parse_offset_out_of_range_negative() {
        assert_eq!(
            "-14:01".parse::<Offset>().unwrap_err().to_string(),
            ParseOffsetError::OutOfRange(-50460).to_string()
        );
    }

    #[test]
    fn test_parse_roundtrip_display() {
        // Pick several offsets randomly to ensure parse(display(o)) == o
        let offsets = vec![
            Offset { seconds: 0 },
            Offset { seconds: 28800 },
            Offset { seconds: -18000 },
            Offset { seconds: 19800 },
            Offset { seconds: -34200 },
            Offset { seconds: 20700 },
            Offset {
                seconds: 9 * 60 + 21,
            },
            Offset {
                seconds: -(5 * 3600 + 30 * 60 + 15),
            },
            Offset { seconds: 50400 },
            Offset { seconds: -50400 },
        ];

        for original in offsets {
            let s = original.to_string();
            let parsed: Offset = s.parse().unwrap();
            assert_eq!(
                parsed.seconds(),
                original.seconds(),
                "failed for offset {}",
                s
            );
        }
    }
}
