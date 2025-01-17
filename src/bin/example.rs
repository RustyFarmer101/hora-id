use std::thread::sleep;
use std::time::Duration;
use tuid::TuidGenerator;

fn main() {
    let mut generator = TuidGenerator::new(1).unwrap();
    for _i in 0..20 {
        let id = generator.next();
        sleep(Duration::from_millis(1));
        println!("ID {:?}", id.to_string());
    }
}
