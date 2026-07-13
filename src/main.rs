use std::{task::Poll::{self, Pending, Ready}, time::Duration};
use stick::{Controller, Event, Listener};
use tokio::{join, select, sync::oneshot, task::{self, JoinSet, LocalSet}, time::sleep};
use tokio::sync::oneshot::{Receiver, Sender};

type Exit = usize;

struct State {
    listener: Listener,
    controllers: Vec<Controller>,
    rumble: (f32, f32),
}

impl State {
    fn connect(&mut self, controller: Controller) -> Poll<Exit> {
        println!(
            "Connected p{}, id: {:016X}, name: {}",
            self.controllers.len() + 1,
            controller.id(),
            controller.name(),
        );
        self.controllers.push(controller);
        Pending
    }

    fn event(&mut self, id: usize, event: Event) -> Poll<Exit> {
        let player = id + 1;
        println!("p{}: {}", player, event);
        match event {
            Event::Disconnect => {
                self.controllers.swap_remove(id);
            }
            Event::MenuR(true) => return Ready(player),
            Event::ActionA(pressed) => {
                self.controllers[id].rumble(f32::from(u8::from(pressed)));
            }
            Event::ActionB(pressed) => {
                self.controllers[id].rumble(0.5 * f32::from(u8::from(pressed)));
            }
            Event::BumperL(pressed) => {
                self.rumble.0 = f32::from(u8::from(pressed));
                self.controllers[id].rumble(self.rumble);
            }
            Event::BumperR(pressed) => {
                self.rumble.1 = f32::from(u8::from(pressed));
                self.controllers[id].rumble(self.rumble);
            }
            _ => {}
        }
        Pending
    }
}

async fn stick_task() {
    let mut js_listener = Listener::default();

    
    /*
    // Spawn a task to listen for joystick devices
    let listen_task = task::spawn(async move {
        // Listen for joysticks to be added to the system.
        let listener = Listener::default();
        loop {
            let controller = Arc::new(Mutex::new(listener.await));
            js_tasks.spawn(controller);
        }
    });
    */

    println!("Started stick task.");
    let mut js_tasks = LocalSet::new();

    join! ( 
        async {
            let controller = js_listener.await;
            js_tasks.spawn_local(controller);
            println!("Got controller");
        },
        async {  // TODO-DW : Fix this.
            let event = js_tasks.await;
            println!("Event! {event:?}");
        }
    );

    /*
    let mut state = State {
        listener: Listener::default(),
        controllers: Vec::new(),
        rumble: (0.0, 0.0),
    };

    // TODO: Do these things in parallel:
    //   listen for controllers, process with State::connect method.
    //   poll controllers, process with State::event method
    let mut futures = JoinSet::new();
    let handle = futures.spawn(state.listener);           // Listen for new controllers
    let listener_id = handle.id();
    while let Some(Ok((id, result))) = futures.join_next_with_id().await {
        match id {
            listener_id => {
                // Stick listener found a new joystick
                println!("Found a joystick");
                state.controllers.push(controller);
                futures.spawn(controller);
            }
            _ => {
                // joystick event detected
                println!("Got joystick event");
            }
        }
    }
    */

    /*
    let player_id = Loop::new(&mut state)
        .when(|s| &mut s.listener, State::connect)
        .poll(|s| &mut s.controllers, State::event)
        .await;
    */

    println!("ended the session");
}

// TODO: Signal end of the program when user presses 'q'
// (for now, just wait 30 seconds)
// This task is in a 
async fn quit_on_q() {
    sleep(Duration::from_secs(10)).await;
}

#[tokio::main]
async fn main() {
    let local = task::LocalSet::new();

    let t1 = local.run_until(stick_task());
    let t2 = local.run_until(quit_on_q());
    select! {
        _ = t1 => { println!(" Joystick task finished."); },
        _ = t2 => { println!(" 10 second timeout ended it."); },
    }
    println!("All done.");
}
