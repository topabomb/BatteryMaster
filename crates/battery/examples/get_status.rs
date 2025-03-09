use status::Status;
use status::Last;
use std::thread::sleep;
use std::time::Duration;
use battery::battery_status;
fn main(){
    let mut rows=battery_status::Status::build().unwrap();
    println!("{rows:#?}");
    sleep(Duration::from_secs(1));
    for row in rows.as_mut_slice() {
        row.last();
    }
    
    println!("{:?}",rows);
    let b=rows.get_mut(0);
    let b=b.unwrap();
    loop{
        b.last();
        println!("{:?}",b);
        sleep(Duration::from_secs(1));
    }
    
}