[![Crates.io]](https://crates.io/crates/emitix)
[![Docs.rs]](https://docs.rs/emitix/)
# Emitix
A Rust library for event-driven programming

## Example
```rust
use emitix::EventManager;

fn main() {
    // Create an event manager
    let mut emitter = EventManager::<()>::new();

    // Add an event listener
    emitter.add("event_name", |event_value: String| {
        println!("Event received: {event_value:?}");
    });

    // Emit an event
    emitter.emit("event_name", String::new("Hello, world!"));
}
```

[Crates.io]: https://img.shields.io/crates/v/emitix?style=for-the-badge
[Docs.rs]: https://img.shields.io/docsrs/emitix?style=for-the-badge
