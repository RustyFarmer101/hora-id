use hora_id::HoraGenerator;
use std::thread::sleep;
use std::time::Duration;

fn main() {
    let mut generator = HoraGenerator::new(1).unwrap();
    for _i in 0..20 {
        let id = generator.next();
        sleep(Duration::from_millis(1));
        println!("ID {:?}", id.to_string());
    }
}
