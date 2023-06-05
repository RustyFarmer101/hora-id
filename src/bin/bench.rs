use std::collections::HashMap;
use std::time::Instant;
use tuid::TuidGenerator;

const SIZE: usize = 10_000_000;

fn main() {
    let mut data = Vec::with_capacity(SIZE);
    let mut generator = TuidGenerator::new(101).expect("Error B123l");

    let mut counter = 0;
    let time = Instant::now();

    loop {
        counter += 1;
        if counter > SIZE {
            break;
        }

        let id = generator.next();
        data.push(id.to_string());
    }

    let done = time.elapsed();

    // Analysis to find duplicates
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
    }
    let mut counter = 0;
    for (key, value) in map.into_iter() {
        if value > 1 {
            counter += 1;
        }
    }

    println!(
        "total {}, unique {}, duplicates {} in {:?}",
        SIZE,
        SIZE - counter,
        counter,
        done
    );
}
