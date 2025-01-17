# HoraID

A time-sorted 8-byte unique ID generator for distributed systems, micro-services and B-Tree databases.

HoraID supports upto 256 unique machines and can theoretically generate 16.7 million unique IDs per second per machine.
That's a total of maximum 4.2 billion IDs per second. However, modern computers are not able to hit this limit. See
the [performance](#performance) section below for more details.

# Advantages

- Keeps track of which machine was used to generate a particular ID
- Takes 50% less space than UUID, ULID or Snowflake ID
- Can be represented as a 64-bit integer or a hexadecimal string
- No need to have a created_at field as the ID stores the timestamp of generation internally in the first few bytes
- Results in Right-only appends in databases using B-Tree indexes such as MySQL, MongoDB, PostgreSQL. In simple terms,
  this gives faster insertion performance than using UUIDs or any other random ID.

# Composition

HoraID has 4 parts:

- 4 byte timestamp high (seconds)
- 1 byte timestamp low (milliseconds)
- 1 byte machine
- 2 bytes of sequence

# Usage

Generate IDs in a distributed system

```no_run
use hora_id::{HoraGenerator, HoraId};

let machine_id = 1; // You'll ideally get this from environment variable or configuration
let mut generator: HoraGenerator = HoraGenerator::new(machine_id).unwrap();

let id: HoraId = generator.next();
println!("{}", id.to_string()); // example: '00cd01daff010002'
println!("{}", id.to_u64()); // example: 57704355272392706
```

Quickly generate a new ID.

```no_run
use hora_id::HoraId;
let id = HoraId::new(None).unwrap();
```

Note: generating a new ID quickly shall be used during debugging or development phase only as it doesn't gaurantee every
ID to be unique when generating 100s of IDs per second.

# Performance

On a Macbook Pro with M1 Max chip, the included benchmark in
`src/bin/bench.rs` produces 7.4 Million IDs per second on a single thread.
Given that the theoretical limit is 16.7 Million IDs per second,
the package will scale well with future CPUs.
In the benchmark example, M1 Max chip only hits 44% of the limit.

To run the benchmark, execute `cargo r --bin bench --release` on your system.