use std::thread::sleep;
use std::time::Duration;
use tuid::{Tuid, TuidGenerator};

fn main() {
    let mut generator = TuidGenerator::new(1).unwrap();
    for i in 0..10 {
        let id = generator.next();
        sleep(Duration::from_millis(100));
        println!("ID {:?}", id.to_u64());
    }
}
