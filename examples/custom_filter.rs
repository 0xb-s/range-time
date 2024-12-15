use chrono::Timelike;
use chrono::{TimeZone, Utc};
use range_time::{TimeRangeBuilder, TimeStep};

fn main() {
    let start = Utc.ymd(2024, 1, 1).and_hms(0, 0, 0);
    let end = Utc.ymd(2024, 1, 1).and_hms(0, 10, 0);

    let range = TimeRangeBuilder::new()
        .start(start)
        .end(end)
        .step(TimeStep::Minute(1))
        .filter(|dt| dt.minute() % 2 == 0)
        .build()
        .unwrap();

    for time in range {
        println!("{}", time);
    }
}
