use std::collections::HashMap;

use serde_json::Value;
use zbus::{Array, Dict, ObjectPath, OwnedVariant, Signature, Str, Structure, Variant};

use super::{json, properties_json};

const CAPTURES: [&str; 6] = [
    include_str!("../../fixtures/bare-metal/networkmanager/manager.json"),
    include_str!("../../fixtures/bare-metal/networkmanager/device-wlan0.json"),
    include_str!("../../fixtures/bare-metal/networkmanager/device-wlan0-wireless.json"),
    include_str!("../../fixtures/bare-metal/networkmanager/ip4-wlan0.json"),
    include_str!("../../fixtures/bare-metal/networkmanager/ip6-wlan0.json"),
    include_str!("../../fixtures/bare-metal/networkmanager/dns-manager.json"),
];

/// What busctl printed as `{"type", "data"}`, made into the value the bus
/// hands zbus.
fn variant(signature: &Signature, data: &Value) -> Variant<'static> {
    let number = || data.as_i64().unwrap();
    let unsigned = || data.as_u64().unwrap();

    match signature {
        Signature::U8 => Variant::U8(u8::try_from(unsigned()).unwrap()),
        Signature::Bool => Variant::Bool(data.as_bool().unwrap()),
        Signature::I16 => Variant::I16(i16::try_from(number()).unwrap()),
        Signature::U16 => Variant::U16(u16::try_from(unsigned()).unwrap()),
        Signature::I32 => Variant::I32(i32::try_from(number()).unwrap()),
        Signature::U32 => Variant::U32(u32::try_from(unsigned()).unwrap()),
        Signature::I64 => Variant::I64(number()),
        Signature::U64 => Variant::U64(unsigned()),
        Signature::F64 => Variant::F64(data.as_f64().unwrap()),
        Signature::Str => Variant::Str(Str::from(data.as_str().unwrap().to_owned())),
        Signature::ObjectPath => {
            Variant::ObjectPath(ObjectPath::try_from(data.as_str().unwrap().to_owned()).unwrap())
        }
        Signature::Variant => Variant::Variant(Box::new(typed(data))),
        Signature::Array(child) => {
            let mut array = Array::new(child);
            for item in data.as_array().unwrap() {
                array.append(variant(child, item)).unwrap();
            }

            Variant::Array(array)
        }
        Signature::Dict { key, value } => {
            let mut dict = Dict::new(key, value);
            for (name, item) in data.as_object().unwrap() {
                let name = variant(key, &Value::String(name.clone()));
                dict.append(name, variant(value, item)).unwrap();
            }

            Variant::Dict(dict)
        }
        Signature::Structure(fields) => {
            let items = data.as_array().unwrap();
            let structure = fields
                .iter()
                .zip(items)
                .fold(Structure::builder(), |structure, (field, item)| {
                    structure.append_field(variant(field, item))
                });

            Variant::Structure(structure.build().unwrap())
        }
        other @ (Signature::Unit | Signature::Signature | Signature::Fd) => {
            panic!("no capture holds a {other}")
        }
    }
}

fn typed(value: &Value) -> Variant<'static> {
    let signature: Signature = value["type"].as_str().unwrap().parse().unwrap();
    variant(&signature, &value["data"])
}

/// busctl's JSON with each `{"type", "data"}` replaced by its data.
fn plain(value: &Value) -> Value {
    match value {
        Value::Object(map) if map.len() == 2 && map.contains_key("type") => plain(&map["data"]),
        Value::Object(map) => Value::Object(
            map.iter()
                .map(|(key, item)| (key.clone(), plain(item)))
                .collect(),
        ),
        Value::Array(items) => Value::Array(items.iter().map(plain).collect()),
        Value::Null | Value::Bool(_) | Value::Number(_) | Value::String(_) => value.clone(),
    }
}

#[test]
fn every_captured_reply_reads_as_busctl_printed_it() {
    for capture in CAPTURES {
        let reply: Value = serde_json::from_str(capture).unwrap();
        let properties = reply["data"][0].as_object().unwrap();

        for (name, value) in properties {
            assert_eq!(json(&typed(value)), plain(value), "{name}");
        }
    }
}

#[test]
fn the_device_list_is_a_list() {
    let reply: Value = serde_json::from_str(CAPTURES[0]).unwrap();
    let devices = json(&typed(&reply["data"][0]["Devices"]));

    assert!(devices.is_array(), "{devices}");
    assert_eq!(devices[0], "/org/freedesktop/NetworkManager/Devices/1");
}

#[test]
fn a_whole_reply_reads_as_one_object_of_plain_values() {
    let reply: Value = serde_json::from_str(CAPTURES[0]).unwrap();
    let properties: HashMap<String, OwnedVariant> = reply["data"][0]
        .as_object()
        .unwrap()
        .iter()
        .map(|(name, value)| (name.clone(), typed(value).try_into_owned().unwrap()))
        .collect();

    assert_eq!(properties_json(&properties), plain(&reply["data"][0]));
}
