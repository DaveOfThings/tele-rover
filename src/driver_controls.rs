use tokio::sync::mpsc;
use crate::robot_system::RobotSystem;
use gilrs::{Axis, ev::{Button, Event, EventType}};



pub struct DriverControls<'a> {
    robot_system: &'a RobotSystem<'a>,
    quit_tx: mpsc::Sender<()>,
    // quit_rx: mpsc::Receiver<()>,
    left_thumb_button: bool,
    left_x: f32,
    left_y: f32,
    right_x: f32,
    quit_rover: bool,
}

impl<'a> DriverControls<'a> {
    pub fn new(robot_system: &'a RobotSystem<'a>, quit_tx: mpsc::Sender<()>) -> DriverControls<'a> {
        // let (quit_tx, quit_rx) = mpsc::channel(1);
        DriverControls {robot_system, quit_tx, left_thumb_button: false, left_x: 0.0, left_y: 0.0, right_x: 0.0, quit_rover: false }
    }

    pub async fn update_robot(&mut self) {

        // Drive or Spin based on whether left thumb button is pressed.
        if self.left_thumb_button {
            // spin
            let turn_speed_rps = self.right_x * self.robot_system.get_max_spin_rps();
            self.robot_system.set_spin_rps(turn_speed_rps).await;
        }
        else {
            // drive
            let curvature = self.right_x * self.robot_system.get_max_curvature();
            let lin_mps = -self.left_y * self.robot_system.get_max_vel_mps();
            self.robot_system.set_drive(lin_mps, curvature).await;
        }

        self.robot_system.set_quit_rover(self.quit_rover).await;
    }

    pub async fn handle(&mut self, e: Event) {
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
            EventType::ButtonPressed(Button::LeftThumb, _code) => {
                // Left joystick button pressed
                self.left_thumb_button = true;
            }
            EventType::ButtonReleased(Button::LeftThumb, _code) => {
                // Left joystick button released
                self.left_thumb_button = false;
            }
            EventType::ButtonPressed(Button::West, _code) => {
                // X button pressed, signal quit to the rover
                self.quit_rover = true;
                println!("Quit rover true.");
            }
            EventType::ButtonReleased(Button::West, _code) => {
                // X button released, stop signalling quit to the rover
                self.quit_rover = false;
                println!("Quit rover false.");
            }
            EventType::AxisChanged(axis, value, _code) => {
                match axis {
                    Axis::RightStickX => {
                        self.right_x = value;

                    },
                    Axis::LeftStickY => {
                        self.left_y = value;
                    },
                    Axis::LeftStickX => {
                        self.left_x = value;
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

        self.update_robot().await;
    }
}
