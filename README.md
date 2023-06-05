# TUID
A time-sorted 8-byte unique ID generator for distributed systems. TUIDs are time-sorted and only 8 bytes long, which is half the length of a UUID and ULID.

TUID supports upto 256 unique machines and generates about 2.5 million IDs per second for a single machine. That's a total of maximum 640 million IDs per second.

# Composition
TUID has 3 parts:
- 4 byte timestamp high
- 1 byte timestamp low
- 3 bytes of randomness

# Usage
Generate IDs in a distributed system
```rust
use tuid::{TuidGenerator, Tuid};

let machine_id = 1;
let mut generator: TuidGenerator = TuidGenerator::new(machine_id).unwrap();

let id: Tuid = generator.next();
println!("{}", id.to_string());
```

Quickly generate a new ID.
```rust
let id = Tuid::new().unwrap();
```
Note: generating a new ID quickly shall be used during debugging or development phase only as it doesn't gaurantee every ID to be unique when generating 100s of IDs per second.

# Performance
On a Macbook Pro with M1 Max chip, the included benchmark in `src/bin/bench.rs` produces 2.44 Million IDs per second on a single thread. Given that the theoretical limit is 2.5 Million IDs per second, modern CPUs are able to come close to this limit. In the benchmark example, M1 Max chip hits 97.6% of the limit.

To run the benchmark, execute `cargo r --bin bench --release` on your system.