use std::time::Duration;
use rumqttc::{MqttOptions, AsyncClient, QoS, EventLoop};
use serde::Deserialize;
use std::fs;
use tokio::task;

use tokio::time;

use crate::robot_system::CommandState;

// Define a struct that mirrors your YAML structure
#[derive(Debug, Deserialize)]
struct MqttHost {
    host: String,
    port: u16,
    user: String,
    password: String,
}

pub struct RobotLink {
    client: AsyncClient,
    eventloop: EventLoop,
}

impl RobotLink {
    pub fn new() -> RobotLink {
        let filename = "mqtt-server.yml";
        let contents = fs::read_to_string(filename)
            .expect("Could not read mqtt-host.yml file");
        let host_info: MqttHost = serde_yaml::from_str(&contents)
            .expect("Could not parse YAML");

        // TODO-DW : Convert from rmqttc example to what tele-rover needs
        let mut mqttoptions = MqttOptions::new("rumqtt-async", host_info.host, host_info.port);
        mqttoptions.set_credentials(host_info.user, host_info.password);
        mqttoptions.set_keep_alive(Duration::from_secs(5));

        println!("RobotLink starting");

        let (client, mut eventloop) = AsyncClient::new(mqttoptions, 10);

        RobotLink { client, eventloop }
    }

    pub async fn run(&self) {

        let result = self.client.subscribe("hello/rumqtt", QoS::AtMostOnce).await;
        match result {
            Ok(_) => {
                println!("Subscribed.");
            }
            Err(_) => {
                println!("Subscribe errored.");
            }
        }

        /*
        task::spawn(async move {
            for i in 0..10 {
                let result = self.client.publish("hello/rumqtt", QoS::AtLeastOnce, false, vec![i; i as usize]).await;
                match result {
                    Ok(_) => { println!("Published."); }
                    Err(_) => { println!("Publish errored."); }
                }

                println!("Published {i}");
                time::sleep(Duration::from_millis(100)).await;
            }
        });
        */

        println!("RobotLink polling event loop");

        while let Ok(notification) = self.eventloop.poll().await {
            println!("Received = {:?}", notification);
        }

        println!("RobotLink ending");
    }

    pub async fn send(&self, command_state: &CommandState) {

    }
}