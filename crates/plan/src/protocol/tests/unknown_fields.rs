use serde::de::DeserializeOwned;
use serde_json::Value;

use super::{Answer, PROTOCOL_VERSION, ProgramAnswer, Request, RequestError};
use crate::sdboot::SdBootMenu;
use crate::{FetchAnswer, GrubMenu, RootAction, ShadowLocks};

/// `json` decodes as `T`, and fails once the object at each of `pointers`
/// holds one field more.
fn refuses_a_surplus_field<T>(json: &str, pointers: &[&str])
where
    T: DeserializeOwned,
{
    let base: Value = serde_json::from_str(json).unwrap();

    serde_json::from_value::<T>(base.clone()).unwrap();

    for pointer in pointers {
        let mut changed = base.clone();
        let Some(Value::Object(object)) = changed.pointer_mut(pointer) else {
            panic!("no object at {pointer:?} in {json}");
        };

        object.insert("surplus".to_owned(), Value::Bool(true));
        assert!(
            serde_json::from_value::<T>(changed).is_err(),
            "a surplus field at {pointer:?} of {json} passed"
        );
    }
}

#[test]
fn a_request_with_a_surplus_field_is_malformed() {
    let line = format!(
        r#"{{"version":{PROTOCOL_VERSION},"mode":"run","steps":[{{"id":1,"action":{{"kind":"enable-units","units":["a.service"]}}}}],"surplus":true}}"#
    );

    assert!(matches!(
        Request::parse(&line),
        Err(RequestError::Malformed(_))
    ));
}

#[test]
fn a_request_refuses_a_surplus_field_in_its_steps() {
    let line = format!(
        r#"{{"version":{PROTOCOL_VERSION},"mode":"run","steps":[{{"id":1,"action":{{"kind":"enable-units","units":["a.service"]}}}}]}}"#
    );

    refuses_a_surplus_field::<Request>(&line, &["", "/steps/0", "/steps/0/action"]);
}

#[test]
fn root_actions_refuse_a_surplus_field_at_every_level() {
    let cases: [(&str, &[&str]); 8] = [
        (
            r#"{"kind":"set-home-folder","user":"alice","change":{"home":"/home/alice2","move_files":true}}"#,
            &["", "/change"],
        ),
        (
            r#"{"kind":"create-user","user":{"name":"alice","full_name":"Alice","shell":null,"groups":[],"home":{"kind":"standard"},"uid":null,"system":false}}"#,
            &["", "/user", "/user/home"],
        ),
        (
            r#"{"kind":"write-config-file","write":{"path":"/etc/X11/xorg.conf","content":"x","original":{"bytes":0,"hash":0}}}"#,
            &["", "/write", "/write/original"],
        ),
        (
            r#"{"kind":"set-grub-settings","settings":[{"key":"timeout","value":5}]}"#,
            &["", "/settings/0"],
        ),
        (
            r#"{"kind":"set-loader-settings","settings":[{"key":"editor","value":false}]}"#,
            &["", "/settings/0"],
        ),
        (
            r#"{"kind":"repair-boot","system":{"root":{"kind":"open","device":"/dev/sda2"},"layout":"plain"},"work":{"kind":"install-bios","disk":"/dev/sda"}}"#,
            &["", "/system", "/system/root", "/work"],
        ),
        (
            r#"{"kind":"install-dkms-module","build":{"module":"vboxhost","version":"7.1.6_OSE","kernel":"6.12.9-1-cachyos-lts"}}"#,
            &["", "/build"],
        ),
        (
            r#"{"kind":"set-locale-enabled","locale":{"locale":"de_DE.UTF-8","charset":"UTF-8"},"enabled":true}"#,
            &["", "/locale"],
        ),
    ];

    for (json, pointers) in cases {
        refuses_a_surplus_field::<RootAction>(json, pointers);
    }
}

#[test]
fn an_answer_refuses_a_surplus_field() {
    refuses_a_surplus_field::<Answer>(r#"{"type":"input","data":"y\n"}"#, &[""]);
    refuses_a_surplus_field::<Answer>(
        r#"{"type":"resize","size":{"columns":80,"rows":24}}"#,
        &[""],
    );
}

#[test]
fn read_answers_refuse_a_surplus_field() {
    refuses_a_surplus_field::<ProgramAnswer>(
        r#"{"result":"ran","output":"","errors":"","code":0,"truncated":false}"#,
        &[""],
    );
    refuses_a_surplus_field::<FetchAnswer>(
        r#"{"result":"text","path":"/etc/X11/xorg.conf","content":"x"}"#,
        &[""],
    );
    refuses_a_surplus_field::<ShadowLocks>(
        r#"{"locks":[{"user":"alice","locked":false}]}"#,
        &["", "/locks/0"],
    );
    refuses_a_surplus_field::<GrubMenu>(
        r#"{"entries":[{"id":"gnulinux-simple","title":"Garuda","position":"0","submenu":false,"parent":null}]}"#,
        &["", "/entries/0"],
    );
    refuses_a_surplus_field::<SdBootMenu>(
        r#"{"settings":{"timeout":null,"default":null,"editor":null,"auto_entries":null,"auto_firmware":null},"entries":[{"id":"garuda.efi","title":"Garuda","version":null,"uname":null,"sort_key":null,"kind":"uki"}],"skipped":0,"cut":0}"#,
        &["", "/settings", "/entries/0"],
    );
}
