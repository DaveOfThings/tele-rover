use serde::Serialize;
use tokio::time;
use tokio::sync::Mutex;
use std::time::Duration;
use std::f32::consts::PI;

use crate::RobotLink;

const MAX_SPEED_MPS: f32 = 2.0;
const MAX_SPIN_RPS: f32 = PI/2.0;   // 90 degrees per second.
const MAX_CURVATURE: f32 = 3.0;     // radians per meter

#[derive(Clone, Copy, Default, Debug, Serialize)]
pub struct DriveSpeed {
    lin_mps: f32,
    curvature: f32,
}

#[derive(Clone, Copy, Default, Debug, Serialize)]
pub struct SpinRate {
    spin_rps: f32,
}

#[derive(Clone, Copy, Debug, Serialize)]
pub enum RobotVel {
    Drive(DriveSpeed),
    Spin(SpinRate),
}

impl RobotVel {
    fn default() -> RobotVel {
        RobotVel::Drive( DriveSpeed { lin_mps: 0.0, curvature: 0.0 })
    }
}

#[derive(Clone, Copy, Debug, Serialize)]
pub enum CommandState {
    Disabled,
    Teleop(RobotVel),
}

pub struct RobotSystem<'a> {
    link: &'a RobotLink,
    command_state: Mutex<CommandState>,
}

impl<'a> RobotSystem<'a> {
    pub fn new(link: &'a RobotLink) -> RobotSystem<'a> {
        let command_state = CommandState::Disabled;
        RobotSystem { link, command_state: Mutex::new(command_state) }
    }

    pub async fn run(&self) {
        let mut interval = time::interval(Duration::from_millis(20));
        loop {
            interval.tick().await;

            let state = self.command_state.lock().await;
            self.link.send(&state).await;
        }
    }

    pub async fn toggle_active(&self) {
        let mut cs = self.command_state.lock().await;
        *cs = match *cs {
            CommandState::Disabled => CommandState::Teleop(RobotVel::default()),
            CommandState::Teleop(_) => CommandState::Disabled,
        };

        println!("Toggled active state to {:?}", *cs);
    }

    pub fn get_max_vel_mps(&self) -> f32 {
        MAX_SPEED_MPS
    }

    pub fn get_max_curvature(&self) -> f32 {
        MAX_CURVATURE
    }

    pub fn get_max_spin_rps(&self) -> f32 {
        MAX_SPIN_RPS
    }

    pub async fn set_drive(&self, lin_mps: f32, curvature: f32) {
        let mut cs = self.command_state.lock().await;
        match *cs {
            CommandState::Disabled => {
                *cs = CommandState::Disabled;
                // println!("It's disabled");
            }
            CommandState::Teleop(_v) => {
                *cs = CommandState::Teleop( RobotVel::Drive(DriveSpeed { lin_mps, curvature } ));
                // println!("It's teleop, drive");
            }
        }
    }

    pub async fn set_spin_rps(&self, spin_rps: f32) {
        let mut cs = self.command_state.lock().await;
        match *cs {
            CommandState::Disabled => {
                *cs = CommandState::Disabled;
                // println!("It's disabled");
            }
            CommandState::Teleop(_v) => {
                *cs = CommandState::Teleop(RobotVel::Spin(SpinRate { spin_rps } ));
                // println!("It's teleop, spin");
            }
        }
    }
}
