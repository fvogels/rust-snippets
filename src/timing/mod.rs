use std::time::Duration;

pub fn measure<T, F: FnMut() -> T>(mut f: F) -> (T, Duration) {
    let before = std::time::Instant::now();
    let result = f();
    let duration = before.elapsed();

    (result, duration)
}
