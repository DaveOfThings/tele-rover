use tokio::time;
use std::time::Duration;

use gilrs::{GamepadId, Gilrs};
use crate::DriverControls;

pub struct JsManager<'a> {
    driver_id: Option<GamepadId>,
    driver: &'a DriverControls<'a>,
}

impl<'a> JsManager<'a> {
    pub fn new(driver: &'a DriverControls<'a>) -> JsManager<'a> {
        JsManager { driver_id: None, driver }
    }

    pub async fn run(&mut self) {
        let mut interval = time::interval(Duration::from_millis(10));
        let mut gilrs = Gilrs::new().unwrap();

        // poll Gilrs
        // Iterate over all connected gamepads
        for (_id, gamepad) in gilrs.gamepads() {
            println!("{} is {:?}", gamepad.name(), gamepad.power_info());
            if self.driver_id.is_none() {
                println!("Assigning gamepad {} to driver.", gamepad.id());
                self.driver_id = Some(gamepad.id());
            }
        }

        loop {
            interval.tick().await;

            // Examine new events
            while let Some(e) = gilrs.next_event() {
                // TODO : Recognize when driver controller is plugged in after startup.

                if self.driver_id == Some(e.id) {
                    // Route this event to the driver controller
                    self.driver.handle(e).await;
                }
            }
        }
    }
}
