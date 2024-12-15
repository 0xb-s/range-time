use chrono::{TimeZone, Utc};
use range_time::{TimeRangeBuilder, TimeStep};

fn main() {
    let start = Utc.ymd(2024, 1, 1).and_hms(0, 0, 0);
    let end = Utc.ymd(2024, 1, 2).and_hms(0, 0, 0);

    let range = TimeRangeBuilder::new()
        .start(start)
        .end(end)
        .step(TimeStep::Hour(6))
        .build()
        .unwrap();

    for time in range {
        println!("{}", time);
    }
}
