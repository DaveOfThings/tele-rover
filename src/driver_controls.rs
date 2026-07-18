use tokio::sync::mpsc;
use crate::robot_system::RobotSystem;
use gilrs::{Axis, ev::{Button, Event, EventType}};



pub struct DriverControls<'a> {
    robot_system: &'a RobotSystem<'a>,
    quit_tx: mpsc::Sender<()>,
    // quit_rx: mpsc::Receiver<()>,
}

impl<'a> DriverControls<'a> {
    pub fn new(robot_system: &'a RobotSystem<'a>, quit_tx: mpsc::Sender<()>) -> DriverControls<'a> {
        // let (quit_tx, quit_rx) = mpsc::channel(1);
        DriverControls {robot_system, quit_tx }
    }

    pub async fn handle(&self, e: Event) {
        match e.event {
            EventType::ButtonPressed(Button::Select, _code) => {
                // "B" button pressed
                println!("TODO: Toggle Active/Inactive.");
                self.robot_system.toggle_active().await;
            }
            EventType::ButtonPressed(Button::Start, _code) => {
                // "B" button pressed
                println!("Exiting.");

                let _ = self.quit_tx.send(()).await;
            }
            EventType::AxisChanged(axis, value, _code) => {
                match axis {
                    Axis::RightStickX => {
                        // Adjust turning speed
                        // Joystick right should turn right, which is negative radians per sec in NED
                        let turn_speed_rps = -value * self.robot_system.get_max_ang_vel_rps();
                        self.robot_system.set_ang_vel_rps(turn_speed_rps).await;
                    },
                    Axis::LeftStickY => {
                        // Adjust linear speed
                        let speed_mps = value * self.robot_system.get_max_vel_mps();
                        self.robot_system.set_lin_vel_mps(speed_mps).await;
                    },
                    _ => {
                        // Ignore left X and right Y
                    }
                }
            }
            _ => {
                println!("Ignored {:?}", e);
            }
        }
    }
}
