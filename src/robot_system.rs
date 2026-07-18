use serde::Serialize;
use tokio::time;
use tokio::sync::Mutex;
use std::time::Duration;

use crate::RobotLink;

const MAX_TURN_SPEED_RPS: f32 = 4.0;
const MAX_SPEED_MPS: f32 = 2.0;

#[derive(Clone, Copy, Default, Debug, Serialize)]
pub struct RobotVel {
    lin_mps: f32,
    ang_rps: f32,
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

            // TODO: Dare Mighty Things
            // TODO: Send command to the robot link
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

    pub fn get_max_ang_vel_rps(&self) -> f32 {
        MAX_TURN_SPEED_RPS
    }

    pub async fn set_lin_vel_mps(&self, vel_mps: f32) {
        let mut cs = self.command_state.lock().await;
        match *cs {
            CommandState::Disabled => {
                *cs = CommandState::Disabled;
                println!("It's disabled");
            }
            CommandState::Teleop(v) => {
                *cs = CommandState::Teleop(RobotVel { lin_mps: vel_mps, ang_rps: v.ang_rps } );
                println!("It's teleop");
            }
        }

        println!("Set vel {vel_mps} [m/s]");
    }

    pub async fn set_ang_vel_rps(&self, ang_vel_rps: f32) {
        let mut cs = self.command_state.lock().await;
        match *cs {
            CommandState::Disabled => {
                *cs = CommandState::Disabled;
                println!("It's disabled");
            }
            CommandState::Teleop(v) => {
                *cs = CommandState::Teleop(RobotVel { lin_mps: v.lin_mps, ang_rps: ang_vel_rps } );
                println!("It's teleop");
            }
        }

        println!("Set ang vel {ang_vel_rps} [m/s]");
    }
}
