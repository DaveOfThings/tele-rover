use std::time::Duration;
use std::error::Error;
use gilrs::{Gilrs, Button};
use tokio::{select, time, task};
use rumqttc::{MqttOptions, AsyncClient, QoS};
use serde::Deserialize;
use std::fs;


// Task to open and read game controller
async fn read_js() {
    let mut interval = time::interval(Duration::from_millis(10));
    let mut gilrs = Gilrs::new().unwrap();

    // poll Gilrs
    // Iterate over all connected gamepads
    for (_id, gamepad) in gilrs.gamepads() {
        println!("{} is {:?}", gamepad.name(), gamepad.power_info());
    }

    let mut active_gamepad = None;

    loop {
        interval.tick().await;

        // Examine new events
        while let Some(e) = gilrs.next_event() {
            println!("{:?} New event from {}: {:?}", e.time, e.id, e.event);
                active_gamepad = Some(e.id);
        }

        // You can also use cached gamepad state
        if let Some(gamepad) = active_gamepad.map(|id| gilrs.gamepad(id)) {
            if gamepad.is_pressed(Button::South) {
                println!("Button South is pressed (XBox - A, PS - X)");
            }
        }

        //println!("Tick");
    }

}

// Define a struct that mirrors your YAML structure
#[derive(Debug, Deserialize)]
struct MqttHost {
    host: String,
    port: u16,
    user: String,
    password: String,
}

// Task to manage MQTT connection
async fn manage_mqtt() {
    let filename = "mqtt-server.yml";
    let contents = fs::read_to_string(filename)
        .expect("Could not read mqtt-host.yml file");
    let host_info: MqttHost = serde_yaml::from_str(&contents)
        .expect("Could not parse YAML");

    // TODO-DW : Convert from rmqttc example to what tele-rover needs
    let mut mqttoptions = MqttOptions::new("rumqtt-async", host_info.host, host_info.port);
    mqttoptions.set_credentials(host_info.user, host_info.password);
    mqttoptions.set_keep_alive(Duration::from_secs(5));

    let (mut client, mut eventloop) = AsyncClient::new(mqttoptions, 10);
    client.subscribe("hello/rumqtt", QoS::AtMostOnce).await.unwrap();

    task::spawn(async move {
        for i in 0..10 {
            client.publish("hello/rumqtt", QoS::AtLeastOnce, false, vec![i; i as usize]).await.unwrap();
            println!("Published {i}");
            time::sleep(Duration::from_millis(100)).await;
        }
    });

    while let Ok(notification) = eventloop.poll().await {
        println!("Received = {:?}", notification);
    }
}

#[tokio::main()]
async fn main() {
    select! {
        _ = read_js() => { 
            println!("read_js ended.");
        },
        _ = manage_mqtt() => {
            println!("mqtt ended.")
        }
    };

    println!("All done.");
}
