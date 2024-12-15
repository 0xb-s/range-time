# range-time

A library for iterating ove time ranges. 

## Example usage

```rust

let start = Utc.ymd(2024, 1, 1).and_hms(0, 0, 0);
let end = Utc.ymd(2024, 1, 1).and_hms(0, 10, 0);

let range = TimeRangeBuilder::new()
    .start(start)
    .end(end)
    .step(TimeStep::Minute(2))
    .build()
    .unwrap();

for datetime in range {
    println!("{}", datetime);
}
```

## License

This project is licensed under the MIT License.
