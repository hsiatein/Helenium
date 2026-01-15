use std::time::Duration;

use anyhow::Context;
use anyhow::Result;
use chrono::DateTime;
use chrono::Datelike;
use chrono::Days;
use chrono::FixedOffset;
use chrono::NaiveDate;
use chrono::NaiveDateTime;
use chrono::NaiveTime;
use chrono::TimeDelta;
use chrono::TimeZone;
use chrono::Utc;
use chrono::Weekday;
use itertools::iproduct;
use serde::Deserialize;
use serde::Serialize;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TriggerTime {
    Once {
        time: DateTime<FixedOffset>,
    },
    Interval {
        anchor: DateTime<FixedOffset>,
        interval_minutes: u64,
    },
    Daily {
        time: NaiveTime,
    },
    Weekly {
        weekday: chrono::Weekday,
        time: NaiveTime,
    },
    Monthly {
        day: u8,
        time: NaiveTime,
    },
}

impl TriggerTime {
    pub fn from_once(rfc3339_str: &str) -> Result<Vec<TriggerTime>> {
        let strs = rfc3339_str.split(",").collect::<Vec<&str>>();
        let triggers = strs
            .iter()
            .filter_map(|str| DateTime::parse_from_rfc3339(str).ok())
            .map(|time| TriggerTime::Once { time })
            .collect::<Vec<TriggerTime>>();
        if strs.len() != triggers.len() {
            Err(anyhow::anyhow!("RFC3339 时间格式错误"))
        } else {
            Ok(triggers)
        }
    }

    pub fn from_interval(interval_minutes: u64, offset: &FixedOffset) -> Result<TriggerTime> {
        if interval_minutes == 0 {
            anyhow::bail!("Interval 必须 >= 1 分钟");
        }
        let anchor = Utc::now().with_timezone(offset);
        let interval = TriggerTime::Interval {
            anchor,
            interval_minutes,
        };
        Ok(interval)
    }

    pub fn from_cron(cron_str: &str) -> Result<Vec<TriggerTime>> {
        let fields: Vec<&str> = cron_str.split_whitespace().collect();
        if fields.len() != 5 {
            return Err(anyhow::anyhow!("Cron 表达式格式错误，应为 5 个字段"));
        }
        let raw_minutes: Vec<&str> = fields
            .get(0)
            .context("minute 获取失败")?
            .split(",")
            .collect();
        let minutes: Vec<u32> = raw_minutes
            .iter()
            .filter_map(|str| str.parse().ok())
            .collect();
        if raw_minutes.len() != minutes.len() {
            return Err(anyhow::anyhow!("Cron 表达式中的分钟字段格式错误"));
        }
        let raw_hours: Vec<&str> = fields.get(1).context("hour 获取失败")?.split(",").collect();
        let hours: Vec<u32> = raw_hours
            .iter()
            .filter_map(|str| str.parse().ok())
            .collect();
        if raw_hours.len() != hours.len() {
            return Err(anyhow::anyhow!("Cron 表达式中的小时字段格式错误"));
        }
        let nts: Vec<NaiveTime> = iproduct!(hours, minutes)
            .filter_map(|(h, m)| NaiveTime::from_hms_opt(h, m, 0))
            .collect();
        if nts.is_empty() {
            return Err(anyhow::anyhow!("Cron 表达式中的小时或分钟字段格式错误"));
        }
        let dom_str = *fields.get(2).context("day of month 获取失败")?;
        if *fields.get(3).context("month 获取失败")? != "*" {
            return Err(anyhow::anyhow!("Cron 表达式中的月份字段必须为 '*'"));
        }
        let dow_str = *fields.get(4).context("day of week 获取失败")?;
        let triggers: Vec<TriggerTime> = match (dom_str, dow_str) {
            ("*", "*") => nts
                .into_iter()
                .map(|nt| TriggerTime::Daily { time: nt })
                .collect(),
            (dom_str, dow_str) => {
                let doms = dom_str
                    .split(",")
                    .filter_map(|s| s.parse::<u8>().ok())
                    .collect::<Vec<u8>>();
                let dows = dow_str
                    .split(",")
                    .filter_map(|s| match s.parse::<u8>() {
                        Ok(0) => Some(chrono::Weekday::Sun),
                        Ok(1) => Some(chrono::Weekday::Mon),
                        Ok(2) => Some(chrono::Weekday::Tue),
                        Ok(3) => Some(chrono::Weekday::Wed),
                        Ok(4) => Some(chrono::Weekday::Thu),
                        Ok(5) => Some(chrono::Weekday::Fri),
                        Ok(6) => Some(chrono::Weekday::Sat),
                        Ok(7) => Some(chrono::Weekday::Sun),
                        _ => None,
                    })
                    .collect::<Vec<chrono::Weekday>>();
                let mut trigger_m: Vec<TriggerTime> = doms
                    .into_iter()
                    .flat_map(|dom| {
                        nts.iter()
                            .cloned()
                            .map(|nt| TriggerTime::Monthly { day: dom, time: nt })
                            .collect::<Vec<TriggerTime>>()
                    })
                    .collect();
                let trigger_w: Vec<TriggerTime> = dows
                    .into_iter()
                    .flat_map(|dow| {
                        nts.iter()
                            .cloned()
                            .map(|nt| TriggerTime::Weekly {
                                weekday: dow,
                                time: nt,
                            })
                            .collect::<Vec<TriggerTime>>()
                    })
                    .collect();
                trigger_m.extend(trigger_w);
                trigger_m
            }
        };
        Ok(triggers)
    }

    pub fn next_trigger(&self, offset: &FixedOffset) -> Result<DateTime<FixedOffset>> {
        let now: DateTime<FixedOffset> = Utc::now().with_timezone(offset);
        match self {
            TriggerTime::Once { time } => {
                if now > *time {
                    Err(anyhow::anyhow!("Once 触发时间已过"))
                } else {
                    Ok(*time)
                }
            }
            TriggerTime::Interval {
                anchor,
                interval_minutes,
            } => Ok(next_interval_trigger(anchor, *interval_minutes)?),
            TriggerTime::Daily { time } => {
                let today = now.date_naive();
                let scheduled_naive = NaiveDateTime::new(today, *time);
                let scheduled_today = offset
                    .from_local_datetime(&scheduled_naive)
                    .single()
                    .context("FixedOffset 转换失败（理论上不该发生）")?;
                Ok(next_interval_trigger(&scheduled_today, 24 * 60)?)
            }
            TriggerTime::Weekly { weekday, time } => {
                let today = now.date_naive();
                let days_ahead: u32 = (weekday.num_days_from_monday() + 7
                    - today.weekday().num_days_from_monday())
                    % 7;
                let candidate_date = today
                    .checked_add_days(Days::new(days_ahead as u64))
                    .context("Weekly 计算候选日期失败")?;
                let candidate_naive = NaiveDateTime::new(candidate_date, *time);
                let candidate = offset
                    .from_local_datetime(&candidate_naive)
                    .single()
                    .context("Weekly FixedOffset 转换失败（理论上不该发生）")?;
                Ok(next_interval_trigger(&candidate, 7 * 24 * 60)?)
            }
            TriggerTime::Monthly { day, time } => {
                if *day == 0 {
                    return Err(anyhow::anyhow!("Monthly day 必须 >= 1"));
                }
                let today = now.date_naive();
                let year = today.year();
                let month = today.month();
                let candidate = monthly_candidate(year, month, *day, *time, offset)?;
                if candidate > now {
                    Ok(candidate)
                } else {
                    let (next_year, next_month) = if month == 12 {
                        (year + 1, 1)
                    } else {
                        (year, month + 1)
                    };
                    Ok(monthly_candidate(
                        next_year, next_month, *day, *time, offset,
                    )?)
                }
            }
        }
    }

    pub fn to_string(&self) -> String {
        match self {
            TriggerTime::Once { time } => {
                format!("单次 {}", time.format("%Y-%m-%d %H:%M:%S"))
            }
            TriggerTime::Interval {
                anchor,
                interval_minutes,
            } => {
                format!(
                    "间隔 {} 分钟, 锚点 {}",
                    interval_minutes,
                    anchor.format("%Y-%m-%d %H:%M:%S")
                )
            }
            TriggerTime::Daily { time } => {
                format!("每日 {}", time.format("%H:%M:%S"))
            }
            TriggerTime::Weekly { weekday, time } => {
                let day = match *weekday {
                    Weekday::Mon => "周一",
                    Weekday::Tue => "周二",
                    Weekday::Wed => "周三",
                    Weekday::Thu => "周四",
                    Weekday::Fri => "周五",
                    Weekday::Sat => "周六",
                    Weekday::Sun => "周日",
                };
                format!("每{} {}", day, time.format("%H:%M:%S"))
            }
            TriggerTime::Monthly { day, time } => {
                format!("每月 {} 日 {}", day, time.format("%H:%M:%S"))
            }
        }
    }
}

fn last_day_of_month(year: i32, month: u32) -> Result<u32> {
    let (next_year, next_month) = if month == 12 {
        (year + 1, 1)
    } else {
        (year, month + 1)
    };
    let first_next = NaiveDate::from_ymd_opt(next_year, next_month, 1)
        .context("Monthly 计算下个月第一天失败")?;
    let last = first_next
        .checked_sub_days(Days::new(1))
        .context("Monthly 计算本月最后一天失败")?;
    Ok(last.day())
}

fn monthly_candidate(
    year: i32,
    month: u32,
    day: u8,
    time: NaiveTime,
    offset: &FixedOffset,
) -> Result<DateTime<FixedOffset>> {
    let last_day = last_day_of_month(year, month)?;
    let candidate_day = std::cmp::min(day as u32, last_day);
    let date =
        NaiveDate::from_ymd_opt(year, month, candidate_day).context("Monthly 计算候选日期失败")?;
    let naive = NaiveDateTime::new(date, time);
    offset
        .from_local_datetime(&naive)
        .single()
        .context("Monthly FixedOffset 转换失败（理论上不该发生）")
}

fn next_interval_trigger(
    anchor: &DateTime<FixedOffset>,
    interval_minutes: u64,
) -> Result<DateTime<FixedOffset>> {
    let now = Utc::now().with_timezone(anchor.offset());
    if now < *anchor {
        return Ok(*anchor);
    }
    let interval = if interval_minutes > 0 {
        interval_minutes as i64
    } else {
        return Err(anyhow::anyhow!("Interval 必须大于 0"));
    };
    let t = (now - anchor).num_minutes() / interval;
    Ok(*anchor + TimeDelta::minutes((t + 1) * interval))
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduledTask {
    pub description: String,
    pub triggers: Vec<TriggerTime>,
    pub offset: i32,
    pub next_trigger: Option<DateTime<FixedOffset>>,
}

impl ScheduledTask {
    pub fn from_once(description: String, offset: i32, once_str: &str) -> Result<ScheduledTask> {
        let triggers = TriggerTime::from_once(once_str)?;
        let mut task = ScheduledTask {
            description,
            triggers,
            offset: offset,
            next_trigger: None,
        };
        task.update_next_trigger();
        Ok(task)
    }

    pub fn from_interval(
        description: String,
        offset: i32,
        interval_str: u64,
    ) -> Result<ScheduledTask> {
        let triggers = vec![TriggerTime::from_interval(
            interval_str,
            &FixedOffset::east_opt(offset).context("解析 offset 失败")?,
        )?];
        let mut task = ScheduledTask {
            description,
            triggers,
            offset: offset,
            next_trigger: None,
        };
        task.update_next_trigger();
        Ok(task)
    }

    pub fn from_cron(description: String, offset: i32, cron_str: &str) -> Result<ScheduledTask> {
        let triggers = TriggerTime::from_cron(cron_str)?;
        let mut task = ScheduledTask {
            description,
            triggers,
            offset: offset,
            next_trigger: None,
        };
        task.update_next_trigger();
        Ok(task)
    }

    pub fn is_ready(&self) -> bool {
        match self.next_trigger {
            Some(next_trigger) => next_trigger <= Utc::now().with_timezone(&self.offset()),
            None => false,
        }
    }

    pub async fn ready(&self) -> Result<()> {
        match self.next_trigger {
            Some(next_trigger) => {
                let dt =
                    (next_trigger - Utc::now().with_timezone(&self.offset())).num_milliseconds();
                let dt = if dt <= 0 {
                    return Ok(());
                } else {
                    dt as u64
                };
                tokio::time::sleep(Duration::from_millis(dt + 1)).await;
                Ok(())
            }
            None => Err(anyhow::anyhow!("没有下次运行时间")),
        }
    }

    pub fn update_next_trigger(&mut self) {
        let mut min_next: Option<DateTime<FixedOffset>> = None;
        let offset = self.offset();
        self.triggers.retain(|t| match t.next_trigger(&offset) {
            Ok(next) => {
                min_next = Some(match min_next {
                    Some(cur) => cur.min(next),
                    None => next,
                });
                true
            }
            Err(_) => false,
        });
        self.next_trigger = min_next;
    }

    fn offset(&self) -> FixedOffset {
        FixedOffset::east_opt(self.offset).expect("offset 设置错误")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_once_parses_all() {
        let offset = FixedOffset::east_opt(8 * 3600).unwrap();
        let t1 = Utc::now().with_timezone(&offset) + TimeDelta::minutes(5);
        let t2 = t1 + TimeDelta::minutes(10);
        let input = format!("{},{}", t1.to_rfc3339(), t2.to_rfc3339());
        let triggers = TriggerTime::from_once(&input).unwrap();
        assert_eq!(triggers.len(), 2);
        match triggers[0] {
            TriggerTime::Once { time } => assert_eq!(time, t1),
            _ => panic!("expected Once"),
        }
        match triggers[1] {
            TriggerTime::Once { time } => assert_eq!(time, t2),
            _ => panic!("expected Once"),
        }
    }

    #[test]
    fn from_once_rejects_invalid() {
        assert!(TriggerTime::from_once("bad").is_err());
    }

    #[test]
    fn from_interval_validates_input() {
        let offset = FixedOffset::east_opt(0).unwrap();
        let trigger = TriggerTime::from_interval(5, &offset).unwrap();
        match trigger {
            TriggerTime::Interval {
                interval_minutes, ..
            } => assert_eq!(interval_minutes, 5),
            _ => panic!("expected Interval"),
        }
        assert!(TriggerTime::from_interval(0, &offset).is_err());
    }

    #[test]
    fn from_cron_parses_daily_weekly_monthly() {
        let daily = TriggerTime::from_cron("0 9 * * *").unwrap();
        assert_eq!(daily.len(), 1);
        match daily[0] {
            TriggerTime::Daily { time } => {
                assert_eq!(time, NaiveTime::from_hms_opt(9, 0, 0).unwrap())
            }
            _ => panic!("expected Daily"),
        }

        let mixed = TriggerTime::from_cron("30 8 15 * 1").unwrap();
        assert_eq!(mixed.len(), 2);
        match mixed[0] {
            TriggerTime::Monthly { day, time } => {
                assert_eq!(day, 15);
                assert_eq!(time, NaiveTime::from_hms_opt(8, 30, 0).unwrap());
            }
            _ => panic!("expected Monthly"),
        }
        match mixed[1] {
            TriggerTime::Weekly { weekday, time } => {
                assert_eq!(weekday, chrono::Weekday::Mon);
                assert_eq!(time, NaiveTime::from_hms_opt(8, 30, 0).unwrap());
            }
            _ => panic!("expected Weekly"),
        }

        assert!(TriggerTime::from_cron("0 9 * *").is_err());
    }

    #[test]
    fn next_interval_trigger_aligns_to_interval() {
        let offset = FixedOffset::east_opt(0).unwrap();
        let now_before = Utc::now().with_timezone(&offset);
        let anchor = now_before - TimeDelta::minutes(7);
        let interval = 5;
        let next = next_interval_trigger(&anchor, interval).unwrap();
        assert!(next >= now_before);
        assert!(next < now_before + TimeDelta::minutes(interval as i64 + 1));
        let diff = (next - anchor).num_minutes();
        assert_eq!(diff.rem_euclid(interval as i64), 0);
    }

    #[test]
    fn test_next_interval_trigger() {
        let offset = FixedOffset::east_opt(0).unwrap();
        let now_before = Utc::now().with_timezone(&offset);
        let anchor = now_before;
        let interval = 5;
        let next = next_interval_trigger(&anchor, interval).unwrap();
        assert!(next >= now_before);
    }

    #[test]
    fn next_trigger_once_works() {
        let offset = FixedOffset::east_opt(0).unwrap();
        let future = Utc::now().with_timezone(&offset) + TimeDelta::minutes(2);
        let trigger = TriggerTime::Once { time: future };
        assert_eq!(trigger.next_trigger(&offset).unwrap(), future);

        let past = Utc::now().with_timezone(&offset) - TimeDelta::minutes(2);
        let trigger = TriggerTime::Once { time: past };
        assert!(trigger.next_trigger(&offset).is_err());
    }

    #[test]
    fn next_trigger_daily_weekly_monthly() {
        let offset = FixedOffset::east_opt(0).unwrap();
        let now = Utc::now().with_timezone(&offset);
        let target = now + TimeDelta::seconds(20);
        let time = target.time();

        let daily = TriggerTime::Daily { time };
        let daily_next = daily.next_trigger(&offset).unwrap();
        let daily_date = if time > now.time() {
            now.date_naive()
        } else {
            now.date_naive().checked_add_days(Days::new(1)).unwrap()
        };
        let daily_expected = offset
            .from_local_datetime(&NaiveDateTime::new(daily_date, time))
            .single()
            .unwrap();
        assert_eq!(daily_next, daily_expected);

        let weekly = TriggerTime::Weekly {
            weekday: now.weekday(),
            time,
        };
        let weekly_next = weekly.next_trigger(&offset).unwrap();
        let weekly_date = if time > now.time() {
            now.date_naive()
        } else {
            now.date_naive().checked_add_days(Days::new(7)).unwrap()
        };
        let weekly_expected = offset
            .from_local_datetime(&NaiveDateTime::new(weekly_date, time))
            .single()
            .unwrap();
        assert_eq!(weekly_next, weekly_expected);

        let monthly = TriggerTime::Monthly { day: 31, time };
        let monthly_next = monthly.next_trigger(&offset).unwrap();
        let year = now.year();
        let month = now.month();
        let last_day = last_day_of_month(year, month).unwrap();
        let candidate_day = std::cmp::min(31u32, last_day);
        let candidate_date = NaiveDate::from_ymd_opt(year, month, candidate_day).unwrap();
        let candidate_dt = offset
            .from_local_datetime(&NaiveDateTime::new(candidate_date, time))
            .single()
            .unwrap();
        let monthly_expected = if candidate_dt > now {
            candidate_dt
        } else {
            let (next_year, next_month) = if month == 12 {
                (year + 1, 1)
            } else {
                (year, month + 1)
            };
            let next_last = last_day_of_month(next_year, next_month).unwrap();
            let next_day = std::cmp::min(31u32, next_last);
            let next_date = NaiveDate::from_ymd_opt(next_year, next_month, next_day).unwrap();
            offset
                .from_local_datetime(&NaiveDateTime::new(next_date, time))
                .single()
                .unwrap()
        };
        assert_eq!(monthly_next, monthly_expected);
    }

    #[test]
    fn scheduled_task_ready_and_update() {
        let offset = FixedOffset::east_opt(0).unwrap();
        let now = Utc::now().with_timezone(&offset);
        let mut task = ScheduledTask {
            description: "test".to_string(),
            triggers: vec![
                TriggerTime::Once {
                    time: now + TimeDelta::minutes(60),
                },
                TriggerTime::Interval {
                    anchor: now - TimeDelta::minutes(1),
                    interval_minutes: 1,
                },
            ],
            offset: offset.local_minus_utc(),
            next_trigger: Some(now - TimeDelta::minutes(1)),
        };
        assert!(task.is_ready());
        task.update_next_trigger();
        let far_future = now + TimeDelta::minutes(60);
        assert!(task.next_trigger.unwrap() < far_future);
        task.next_trigger = Some(now + TimeDelta::minutes(1));
        assert!(!task.is_ready());
    }
}
