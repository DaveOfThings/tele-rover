use tokio::time;
use tokio::sync::Mutex;
use std::time::Duration;

use crate::RobotLink;

const MAX_TURN_SPEED_RPS: f32 = 1.0;
const MAX_SPEED_MPS: f32 = 1.0;

pub struct CommandState {
    active: bool,
    vel_mps: f32,
    ang_vel_rps: f32,
}

pub struct RobotSystem<'a> {
    _link: &'a RobotLink,
    command_state: Mutex<CommandState>,
}

impl<'a> RobotSystem<'a> {
    pub fn new(_link: &'a RobotLink) -> RobotSystem<'a> {
        let command_state = CommandState { active:false, vel_mps: 0.0, ang_vel_rps: 0.0};
        RobotSystem { _link, command_state: Mutex::new(command_state) }
    }

    pub async fn run(&self) {
        let mut interval = time::interval(Duration::from_millis(20));
        loop {
            interval.tick().await;

            // TODO: Dare Mighty Things
            // TODO: Send command to the robot link
            self.link.send(&self.command_state).await;
        }
    }

    pub async fn toggle_active(&self) -> bool {
        let mut cs = self.command_state.lock().await;
        cs.active = !cs.active;

        println!("Toggled active state to {}", cs.active);

        cs.active
    }

    pub fn get_max_vel_mps(&self) -> f32 {
        MAX_SPEED_MPS
    }

    pub fn get_max_ang_vel_rps(&self) -> f32 {
        MAX_TURN_SPEED_RPS
    }

    pub async fn set_lin_vel_mps(&self, vel_mps: f32) {
        let mut cs = self.command_state.lock().await;
        cs.vel_mps = vel_mps;

        println!("Set vel {vel_mps} [m/s]");
    }

    pub async fn set_ang_vel_rps(&self, ang_vel_rps: f32) {
        let mut cs = self.command_state.lock().await;
        cs.ang_vel_rps = ang_vel_rps;

        println!("Set ang vel {ang_vel_rps} [rad/s]");
    }
}
