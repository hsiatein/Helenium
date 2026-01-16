use anyhow::Context;
use anyhow::Result;
use chrono::DateTime;
use chrono::Local;
use chrono::TimeZone;
use std::collections::VecDeque;
use std::fmt::Write;

fn nice_ceiling(value: f32) -> f32 {
    if value <= 10.0 {
        return 10.0;
    }
    let magnitude = 10f32.powf(value.log10().floor());
    let normalized = value / magnitude;
    let step = if normalized <= 1.0 {
        1.0
    } else if normalized <= 2.0 {
        2.0
    } else if normalized <= 5.0 {
        5.0
    } else {
        10.0
    };
    step * magnitude
}

pub fn generate_svg_path(
    data: &VecDeque<(DateTime<Local>, usize)>,
    width: f32,
    height: f32,
) -> Result<(String, String, String, String, String)> {
    if data.is_empty() {
        return Ok((
            String::new(),
            "0".into(),
            "0".into(),
            "00:00:00".into(),
            "00:00:00".into(),
        ));
    }

    // 1. Sort by timestamp
    let mut raw_points: Vec<_> = data.iter().collect();
    raw_points.sort_by_key(|(time, _)| time);

    // 2. Deduplicate: Merge points with same timestamp (keep max value)
    // points will store (timestamp_millis, traffic_value)
    let mut points: Vec<(i64, usize)> = Vec::with_capacity(raw_points.len());

    if let Some((first_time, first_val)) = raw_points.first() {
        let mut current_time = first_time.timestamp_millis();
        let mut max_val_for_time = *first_val;

        for (time, val) in raw_points.iter().skip(1) {
            let t_millis = time.timestamp_millis();
            let v = *val;

            if t_millis == current_time {
                // Same timestamp: keep the larger value
                if v > max_val_for_time {
                    max_val_for_time = v;
                }
            } else {
                // New timestamp: push previous and reset
                points.push((current_time, max_val_for_time));
                current_time = t_millis;
                max_val_for_time = v;
            }
        }
        // Push the last accumulated point
        points.push((current_time, max_val_for_time));
    }

    if points.is_empty() {
        return Ok((
            String::new(),
            "0".into(),
            "0".into(),
            "00:00:00".into(),
            "00:00:00".into(),
        ));
    }

    // debug!("BusTraffic Graph: {} points after dedup (raw {})", points.len(), data.len());

    // 3. Calculate Range
    let min_time = points.first().context("时间序列没有元素")?.0;
    let max_time = points.last().context("时间序列没有元素")?.0;
    let time_range = (max_time - min_time).max(1) as f32;

    let max_traffic = points.iter().map(|(_, v)| *v).max().unwrap_or(0);
    let min_val = 0.0;
    let max_val = nice_ceiling(max_traffic as f32);
    let mid_val = max_val * 0.5;
    let val_range = (max_val - min_val).max(1.0);

    let mut path = String::with_capacity(points.len() * 30);

    for (i, (time_millis, val)) in points.iter().enumerate() {
        // 归一化并映射到画布坐标
        // Fix: Subtract integers first to preserve precision. converting a huge timestamp to f32 loses precision.
        let time_diff = (*time_millis - min_time) as f32;
        let v_float = *val as f32;

        let x = (time_diff / time_range) * width;
        // 翻转 Y 轴：数值大 -> 坐标小(上方)
        let y = height * (1.0 - (v_float - min_val) / val_range);

        if i == 0 {
            write!(path, "M {:.1} {:.1}", x, y)?;
        } else {
            write!(path, " L {:.1} {:.1}", x, y)?;
        }
    }

    let min_time_dt = Local.timestamp_millis_opt(min_time).single().context("没有单一时间")?;
    let max_time_dt = Local.timestamp_millis_opt(max_time).single().context("没有单一时间")?;
    let x_start = min_time_dt.format("%H:%M:%S").to_string();
    let x_end = max_time_dt.format("%H:%M:%S").to_string();
    // Use the rounded ceiling for the labels to keep the scale clean.
    let y_max = format!("{}", max_val as usize);
    let y_mid = format!("{}", mid_val.round() as usize);

    Ok((path, y_max, y_mid, x_start, x_end))
}
