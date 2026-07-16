mod robot_link;
mod robot_system;
mod js_manager;
mod driver_controls;

use tokio::select;
use tokio::sync::mpsc;



use robot_link::RobotLink;
use robot_system::RobotSystem;
use js_manager::JsManager;
use driver_controls::DriverControls;


#[tokio::main()]
async fn main() {
    // Create RobotLink
    let robot_link = RobotLink::default();
    let robot_system = RobotSystem::new(&robot_link); 

    let (quit_tx, mut quit_rx) = mpsc::channel(1);

    let driver = DriverControls::new(&robot_system, quit_tx);
    let mut js_manager = JsManager::new(&driver);
    // TODO: gunner controls later.

    // Run all the tasks.  If one quits, the app ends.

    select! {
        _ = quit_rx.recv() => {
            println!("Quit signalled.");
        },
        _ = robot_link.run() => { 
            println!("robot link quit.");
        },
        _ = robot_system.run() => {
            println!("robot system quit.");
        }
        _ = js_manager.run() => {
            println!("Joystick manager quit.");
        }
    };

    println!("All done.");
}
