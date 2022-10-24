use std::collections::HashMap;

use nom::{
    character::complete::{newline, space1},
    combinator::{map_res, verify},
    multi::many1,
    sequence::{delimited, pair, terminated},
    IResult,
};

use crate::error::DmiError;

use super::{
    key_value::{key_value, Dirs, KeyValue},
    values::Value,
};

#[derive(Clone, Debug, PartialEq)]
pub struct State {
    pub name: String,
    pub dirs: Dirs,
    pub frames: u32,
    pub delays: Option<Vec<f32>>,
    pub loop_flag: Option<u32>,
    pub rewind: Option<u32>,
    pub movement: Option<u32>,
    pub hotspot: Option<[f32; 3]>,
    pub unk: Option<HashMap<String, Value>>,
}

impl TryFrom<(KeyValue, Vec<KeyValue>)> for State {
    type Error = DmiError;

    fn try_from((state, kvs): (KeyValue, Vec<KeyValue>)) -> Result<Self, Self::Error> {
        let name = match state {
            KeyValue::State(name) => name,
            _ => unreachable!(),
        };

        let mut dirs = None;
        let mut frames = 1;
        let mut delays = None;
        let mut loop_flag = None;
        let mut rewind = None;
        let mut movement = None;
        let mut hotspot = None;
        let mut unk: Option<HashMap<String, Value>> = None;

        for kv in kvs {
            match kv {
                KeyValue::Dirs(d) => dirs = Some(d),
                KeyValue::Frames(f) => {
                    frames = f;
                }
                KeyValue::Delay(f) => delays = Some(f),
                KeyValue::Loop(do_loop) => loop_flag = Some(do_loop),
                KeyValue::Rewind(do_rewind) => rewind = Some(do_rewind),
                KeyValue::Movement(do_movement) => movement = Some(do_movement),
                KeyValue::Hotspot(h) => {
                    if h.len() == 3 {
                        let mut buf = [0.0; 3];
                        buf.copy_from_slice(&h[0..3]);
                        hotspot = Some(buf);
                    } else {
                        return Err(DmiError::Generic(
                            "Hotspot information was not length 3".to_owned(),
                        ));
                    }
                }
                KeyValue::Unk(key, value) => {
                    if let Some(map) = &mut unk {
                        map.insert(key, value);
                    } else {
                        let mut new_map = HashMap::new();
                        new_map.insert(key, value);
                        unk = Some(new_map);
                    }
                }
                x => {
                    return Err(DmiError::Generic(format!("{:?} not allowed here", x)));
                }
            }
        }

        Ok(State {
            name,
            dirs: dirs.ok_or_else(|| {
                DmiError::Generic("Required field `dirs` was not found".to_owned())
            })?,
            frames,
            delays,
            loop_flag,
            rewind,
            movement,
            hotspot,
            unk,
        })
    }
}

pub fn state(input: &str) -> IResult<&str, State> {
    map_res(
        pair(
            verify(terminated(key_value, newline), |v| {
                matches!(v, super::key_value::KeyValue::State(_))
            }),
            many1(delimited(space1, key_value, newline)),
        ),
        |(state_name, properties)| State::try_from((state_name, properties)),
    )(input)
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn metadata() {
        let description = r#"
state = "duplicate"
    dirs = 1
    frames = 1
"#
        .trim();

        let (_, state) = state(description).unwrap();
        assert_eq!(state.dirs, Dirs::One);
        assert_eq!(state.frames, 1);
        assert_eq!(state.name, "duplicate");
    }

    #[test]
    fn delay() {
        let description = r#"
state = "bluespace_coffee"
    dirs = 1
    frames = 4
    delay = 1,2,5.4,3
state = "..."
"#
        .trim();

        let (tail, state) = state(description).unwrap();
        assert_eq!(tail, r#"state = "...""#);
        assert_eq!(state.dirs, Dirs::One);
        assert_eq!(state.delays, Some(Vec::from([1.0, 2.0, 5.4, 3.0])));
        assert_eq!(state.name, "bluespace_coffee");
    }

    #[test]
    fn fail_delay_without_frames() {
        let description = r#"
state = "bluespace_coffee"
    dirs = 1
    delay = 1,2,5.4,3
state = "..."
        "#
        .trim();

        let x = state(description);
        assert!(matches!(x, Err(_)));
    }
}
