use std::collections::{HashMap, HashSet};
use tuid::TuidGenerator;

const SIZE: usize = 1_000_000;

fn main() {
    let mut data = Vec::with_capacity(SIZE);
    let mut generator = TuidGenerator::new(101).expect("Error B123l");

    let mut counter = 0;
    loop {
        counter += 1;
        if counter > SIZE {
            break;
        }

        let id = generator.generate();
        data.push(id.to_string());
    }

    let mut map: HashMap<String, u16> = HashMap::with_capacity(SIZE);
    for id in data {
        match map.get_mut(&id) {
            None => {
                map.insert(id, 1);
            }
            Some(val) => {
                *val += 1;
            }
        }
        // let counter = map.entry(&id).or_insert(0);
        // *counter += 1;
    }

    let mut counter = 0;

    for (key, value) in map.into_iter() {
        if value > 1 {
            counter += 1;
            println!("{key} -> {value}");
        }
    }

    println!(
        "total {}, unique {}, duplicates {}",
        SIZE,
        SIZE - counter,
        counter
    );
}
