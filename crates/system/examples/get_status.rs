use std::{thread::sleep, time::Duration};

use status::{Status,Last};
use system::{self, system_status};
fn main(){
    let status=system_status::Status::build();
    if let Some(mut status)=status{
        println!("{status:#?}");
        loop {
            status.last();
            println!("{status:?}");
            sleep(Duration::from_secs(1));
        }
    }
    
}