use std::path::Path;
use std::fs::File;
use std::io::BufReader;
use std::io::prelude::*;

use lazy_static::lazy_static;
use regex::Regex;

use crate::aocbase::{AOCResult, AOCError};

lazy_static! {
    static ref STEP_REGEX: Regex = Regex::new(r"^([A-Za-z]+)(=(\d+)|(-))$").unwrap();
}

#[derive(Debug)]
pub enum LensOperation {
    Remove,
    Focus(i32),
}

#[derive(Debug)]
pub struct InitializationStep {
    pub text: String,
    pub operation: LensOperation,
}

impl InitializationStep {

    pub fn load(input: impl AsRef<Path>) -> AOCResult<Vec<Self>> {
        get_strings(input.as_ref())?
            .iter()
            .map(Self::parse)
            .collect::<AOCResult<Vec<Self>>>()
    }

    pub fn parse(s: impl AsRef<str>) -> AOCResult<Self> {
        let cap = STEP_REGEX
            .captures(s.as_ref())
            .ok_or_else(|| AOCError::ParseError(format!("Invalid step: {}", s.as_ref())))?;

        let text = cap.get(1)
            .ok_or_else(|| AOCError::InvalidRegexOperation("Invalid capture group(1)".into()))?
            .as_str();

        if let Some(m) = cap.get(3) {
            return Ok(InitializationStep {
                text: text.to_string(),
                operation: LensOperation::Focus(m.as_str().parse::<i32>()?)
            });
        }
        else if let Some(_) = cap.get(4) {
            return Ok(InitializationStep {
                text: text.to_string(),
                operation: LensOperation::Remove,
            });
        }

        Err(AOCError::ParseError(format!("Invalid initialization step: {}", s.as_ref())))
    }
}

fn get_strings(input: impl AsRef<Path>) -> AOCResult<Vec<String>> {
    let reader = BufReader::new(File::open(input)?);
    Ok(reader
        .lines()
        .nth(0)
        .ok_or_else(|| AOCError::ParseError("Expected a line.".into()))??
        .trim()
        .split(',')
        .map(|s| s.to_string())
        .collect::<Vec<String>>())
}

/*
    Determine the ASCII code for the current character of the string.
    Increase the current value by the ASCII code you just determined.
    Set the current value to itself multiplied by 17.
    Set the current value to the remainder of dividing itself by 256.
*/
pub fn string_hash(input: impl AsRef<str>) -> i32 {
    input
        .as_ref()
        .as_bytes()
        .iter()
        .fold(0, |current, b| ((current + *b as i32) * 17) % 256)
}

#[derive(Debug)]
pub struct Lens {
    pub label: String,
    pub focal_length: i32,
    pub box_id: i32,
}

impl Lens {
    pub fn new(label: String, focal_length: i32) -> Self {
        let box_id = string_hash(&label);
        Self { label, focal_length, box_id }
    }
}


#[derive(Debug)]
pub struct LightBox {
    slots: Vec<Lens>,
}

impl LightBox {
    pub fn new() -> Self {
        Self { slots: Vec::new() }
    }

    pub fn find_slot_id(&self, label: impl AsRef<str>) -> Option<usize> {
        let label = label.as_ref();
        self
            .slots
            .iter()
            .enumerate()
            .find(|(_, light_box)| light_box.label == label)
            .map(|(idx, _)| idx)
    }

    pub fn remove(&mut self, label: impl AsRef<str>) {
        if let Some(id) = self.find_slot_id(label) {
            // This should bubble the item up to the end.
            for pos in id..(self.slots.len() -1) {
                self.slots.swap(pos, pos+1);
            }
            self.slots.pop();
        }
    }

    pub fn add(&mut self, lens: Lens) {
        if let Some(id) = self.find_slot_id(&lens.label) {
            self.slots[id] = lens;
        }
        else {
            self.slots.push(lens);
        }
    }
}

#[derive(Debug)]
pub struct LightBoxes {
    boxes: Vec<LightBox>,
}

impl LightBoxes {
    pub fn new() -> Self {
        Self { boxes: (0..256).map(|_| LightBox::new()).collect() }
    }

    pub fn process(&mut self, init_step: &InitializationStep) {
        match init_step.operation {
            LensOperation::Remove => {
                self.remove(&init_step.text);
            },
            LensOperation::Focus(n) => {
                self.add(&init_step.text, n);
            }
        }
    }

    pub fn remove(&mut self, label: impl AsRef<str>) {
        let label = label.as_ref();
        let box_id = string_hash(label);
        let lens_box = &mut self.boxes[box_id as usize];
        lens_box.remove(label);
    }

    pub fn add(&mut self, label: impl AsRef<str>, focal_length: i32) {
        let label = label.as_ref();
        let box_id = string_hash(label);
        let lens_box = &mut self.boxes[box_id as usize];
        lens_box.add(Lens::new(label.into(), focal_length));
    }

    /*
      To confirm that all of the lenses are installed correctly, add up the focusing power of
      all of the lenses. The focusing power of a single lens is the result of multiplying together:

        One plus the box number of the lens in question.
        The slot number of the lens within the box: 1 for the first lens, 2 for the second lens, and so on.
        The focal length of the lens.

        At the end of the above example, the focusing power of each lens is as follows:

        rn: 1 (box 0) * 1 (first slot) * 1 (focal length) = 1
        cm: 1 (box 0) * 2 (second slot) * 2 (focal length) = 4
        ot: 4 (box 3) * 1 (first slot) * 7 (focal length) = 28
        ab: 4 (box 3) * 2 (second slot) * 5 (focal length) = 40
        pc: 4 (box 3) * 3 (third slot) * 6 (focal length) = 72
     */
    pub fn get_focussing_power(&self) -> i64 {
        let mut focus_power: i64 = 0;

        for (box_id, light_box) in self.boxes.iter().enumerate() {
            for (slot_id, lens) in light_box.slots.iter().enumerate() {
                focus_power += (box_id + 1) as i64 * (slot_id + 1) as i64 * lens.focal_length as i64;
            }
        }

        focus_power
    }
}

pub fn part1(input: impl AsRef<Path>) -> AOCResult<String> {
    Ok(get_strings(input)?
        .iter()
        .map(string_hash)
        .sum::<i32>()
        .to_string())
}

pub fn part2(input: impl AsRef<Path>) -> AOCResult<String> {
    let mut light_boxes = LightBoxes::new();

    let init_steps = InitializationStep::load(input)?;
    for init_step in  &init_steps {
        light_boxes.process(init_step);
    }

    let result = light_boxes.get_focussing_power();

    Ok(result.to_string())
}