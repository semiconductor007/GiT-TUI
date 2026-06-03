//! 时间工具：负责 UTC 时间转换和终端显示格式化。

use chrono::{DateTime, Local, Utc};

use crate::utils::{AppError, Result};

pub const DISPLAY_TIME_FORMAT: &str = "%Y-%m-%d %H:%M";

pub fn format_utc_time(time: &DateTime<Utc>) -> String {
    time.with_timezone(&Local)
        .format(DISPLAY_TIME_FORMAT)
        .to_string()
}

pub fn unix_seconds_to_utc(seconds: i64) -> Result<DateTime<Utc>> {
    DateTime::<Utc>::from_timestamp(seconds, 0)
        .ok_or_else(|| AppError::ParseError(format!("invalid unix timestamp: {seconds}")))
}
