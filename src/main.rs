extern crate mqtt3;
extern crate nanomsg;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;

use std::collections::HashMap;
use std::io;
use std::thread;

use serde_json::Value;

mod mqtt;
mod gateway;

use gateway::{Device, Adapter, Plugin, GatewayBridge, Property};

struct MQTTDevice {
    props: HashMap<String, Value>,
    mqtt: mqtt::MQTT
}

impl MQTTDevice {
    fn new(mqtt: mqtt::MQTT) -> MQTTDevice {
        let mut props = HashMap::new();
        props.insert("on".to_string(), Value::Bool(true));

        MQTTDevice {
            props: props,
            mqtt: mqtt
        }
    }

}

impl Device for MQTTDevice {
    fn set_property(&mut self, property: Property) -> Result<Property, io::Error> {
        println!("set_property {:?}", property);
        self.mqtt.publish_value(&property.name, &property.value)
            .map_err(|_| return io::Error::new(io::ErrorKind::Other, "mqtt3 error"))?;
        self.props.insert(property.name.clone(), property.value.clone());
        Ok(property)
    }

    fn get_properties(&self) -> HashMap<String, Property> {
        let mut props = HashMap::new();
        for (name, value) in &self.props {
            props.insert(name.clone(), Property {
                name: name.clone(),
                value: value.clone()
            });
        }
        props
    }

    fn get_name(&self) -> String {
        "ESP8266 LED".to_string()
    }

    fn get_type(&self) -> String {
        "onOffSwitch".to_string()
    }
}

struct MQTTAdapter {
    devices: HashMap<String, Box<MQTTDevice>>
}

impl MQTTAdapter {
    fn new(id: &str, mqtt: mqtt::MQTT) -> MQTTAdapter {
        let mut devices = HashMap::new();
        let device_id = format!("{}-0", id);
        devices.insert(device_id.to_string(), Box::new(MQTTDevice::new(mqtt)));
        MQTTAdapter {
            devices: devices
        }
    }
}

impl Adapter<MQTTDevice> for MQTTAdapter {
    fn start_pairing(&mut self) -> Result<(), io::Error> {
        println!("start_pairing");
        Ok(())
    }

    fn cancel_pairing(&mut self) -> Result<(), io::Error> {
        println!("cancel_pairing");
        Ok(())
    }

    fn set_property(&mut self, device_id: &str, property: Property) -> Result<Property, io::Error> {
        println!("set_property");
        match self.devices.get_mut(device_id) {
            Some(device) => device.set_property(property),
            None => return Err(io::Error::new(io::ErrorKind::Other, "Device not found"))
        }
    }

    fn get_name(&self) -> String {
        "MQTT Adapter".to_string()
    }

    fn get_devices(&self) -> &HashMap<String, Box<MQTTDevice>> {
        &self.devices
    }
}

fn main() {
    let mut mqtt = mqtt::MQTT::new();
    println!("con: {:?}", mqtt.send_connect().unwrap());
    println!("pub: {:?}", mqtt.publish_value("on", &Value::Bool(true)).unwrap());

    let (mut gateway_bridge, msg_sender, msg_receiver) = GatewayBridge::new("mqtt");
    thread::spawn(move || {
        gateway_bridge.run_forever().unwrap();
    });
    let mut plugin = Plugin::new("mqtt", msg_sender, msg_receiver);
    plugin.add_adapter("mqtt-0", Box::new(MQTTAdapter::new("mqtt-0", mqtt)));
    plugin.run_forever().unwrap();
}
