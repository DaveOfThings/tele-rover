use std::time::Duration;
use rumqttc::{Event, Incoming, Publish};
use rumqttc::{MqttOptions, AsyncClient, QoS, EventLoop};
use serde::Deserialize;
use tokio::sync::Mutex;
use std::fs;
use tokio::select;
use tokio::time::{self};
use std::time::Instant;

use crate::robot_system::CommandState;

#[derive(Debug, Deserialize)]
struct MqttHost {
    host: String,
    port: u16,
    user: String,
    password: String,
}

pub struct LinkState {
    last_heartbeat: std::time::Instant,
}

pub struct RobotLink {
    client: Mutex<AsyncClient>,
    eventloop: Mutex<EventLoop>,
    state: Mutex<LinkState>,
}

impl RobotLink {
    const DEFAULT_FILENAME: &str = "mqtt-server.yml";

    pub fn new(filename: &str) -> RobotLink {
        let contents = match fs::read_to_string(filename) {
            Ok(s) => s,
            Err(_) => {
                println!("Couldn't open config file: {}", filename);
                println!("Using default config file: {}", Self::DEFAULT_FILENAME);
                match fs::read_to_string(Self::DEFAULT_FILENAME) {
                    Ok(s) => s,
                    Err(_) => {
                        panic!("Couldn't open default config file: {}", Self::DEFAULT_FILENAME);
                    },
                }
            },
        };

        let host_info: MqttHost = serde_yml::from_str(&contents)
            .expect("Could not parse YAML");

        let mut mqttoptions = MqttOptions::new("operator", host_info.host, host_info.port);
        mqttoptions.set_credentials(host_info.user, host_info.password);
        mqttoptions.set_keep_alive(Duration::from_secs(5));

        println!("RobotLink starting");
        let (client, eventloop) = AsyncClient::new(mqttoptions, 10);

        let state = LinkState { 
            last_heartbeat: Instant::now(),
        };

        RobotLink { client: Mutex::new(client), eventloop: Mutex::new(eventloop), state: Mutex::new(state) }
    }

    pub async fn handle_msg(&self, msg: &Publish) {
        println!("Received = {:?}", msg);
        match msg.topic.as_str() {
            "robot/heartbeat" => {
                // Got heartbeat from robot
                let mut state = self.state.lock().await;
                state.last_heartbeat = Instant::now();
                println!("rx heartbeat.");
            }
            _ => {
                // Ignoring unrecognized topics
            }
        }
    }

    pub async fn run(&self) {
        {
            let client = self.client.lock().await;

            let _ = client.subscribe("robot/heartbeat", QoS::AtMostOnce).await;  // TODO: Handle errors
            let _ = client.subscribe("robot/status", QoS::AtMostOnce).await; // TODO: Handle errors
        }

        // Task to send heartbeats
        let heartbeat_task = async move {
            let mut interval = time::interval(Duration::from_secs(1));
            loop {
                interval.tick().await;
                {
                    let client = self.client.lock().await;

                    // Send heartbeat
                    // TODO: Handle error
                    let _ = client.publish("controller/heartbeat", QoS::AtLeastOnce, false, "still alive").await;
                    println!("heartbeat tx.");
                }
            }
        };

        // Task to handle MQTT events
        let mqtt_handler = async move {
            // Listen for messages from the robot
            println!("RobotLink polling event loop");
            let mut eventloop = self.eventloop.lock().await;
            loop {
                let status = eventloop.poll().await; 
                match status {
                    Ok(notification) => {
                        match notification {
                            Event::Incoming(Incoming::Publish(msg)) => {
                                self.handle_msg(&msg).await;
                            }
                            Event::Outgoing(_) => {
                                // println!("Don't need outgoing notification.");
                            }
                            Event::Incoming(Incoming::PubAck(_)) => {
                                // println!("Don't need to handle pub acks.");
                            }
                            Event::Incoming(Incoming::PingResp) => {
                                // println!("Don't need to handle ping responses.");
                            }
                            Event::Incoming(Incoming::ConnAck(_conn_ack)) => {
                                println!("ConnAck");
                            }
                            _ => {
                                println!("Ignoring notification {:?}", notification);
                            }
                        }
                    }
                    Err(e) => {
                        println!("Error from eventloop.poll: {e:?}");
                    }
                }
            }
        };

        // Activate these two tasks when run() is awaited.
        // Neither should ever end.  If one does, run() terminates
        select! {
            _ = heartbeat_task => { println!("Stopped sending heartbeat."); },
            _ = mqtt_handler => { println!("MQTT handler quit."); }
        }
        
        println!("RobotLink ending");
    }

    pub async fn send(&self, _command_state: &CommandState) {
        // TODO: Format command_state with YML.

        let client = self.client.lock().await;

        let s = serde_json::to_string(_command_state).unwrap();

        let _ = client.publish("robot/command_state", QoS::AtLeastOnce, false, s).await;
    }
}