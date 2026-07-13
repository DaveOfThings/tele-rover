use std::time::Duration;
use gilrs::{Gilrs, Button};
use tokio::{select, time};



async fn read_js() {
    let mut interval = time::interval(Duration::from_millis(10));
    let mut gilrs = Gilrs::new().unwrap();

    // poll Gilrs
    // Iterate over all connected gamepads
    for (_id, gamepad) in gilrs.gamepads() {
        println!("{} is {:?}", gamepad.name(), gamepad.power_info());
    }

    let mut active_gamepad = None;

    loop {
        interval.tick().await;

        // Examine new events
        while let Some(e) = gilrs.next_event() {
            println!("{:?} New event from {}: {:?}", e.time, e.id, e.event);
                active_gamepad = Some(e.id);
        }

        // You can also use cached gamepad state
        if let Some(gamepad) = active_gamepad.map(|id| gilrs.gamepad(id)) {
            if gamepad.is_pressed(Button::South) {
                println!("Button South is pressed (XBox - A, PS - X)");
            }
        }

        //println!("Tick");
    }

}

#[tokio::main()]
async fn main() {
    select! {
        _ = read_js() => { 
            println!("read_js ended.");
        },
    };

    println!("All done.");
}
