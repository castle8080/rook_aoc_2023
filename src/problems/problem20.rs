use std::collections::HashMap;
use std::collections::VecDeque;
use std::fs::File;
use std::io::BufReader;
use std::io::prelude::*;
use std::path::Path;

use lazy_static::lazy_static;
use regex::Regex;

use crate::aocbase::{AOCResult, AOCError};
use crate::regex_ext::CapturesExt;
use crate::regex_ext::RegexExt;

lazy_static! {
    static ref MODULE_REGEX: Regex = Regex::new(
        r"^\s*([&%])?([a-zA-Z]+) -> ([a-zA-Z, ]+?)\s*$"
    ).unwrap();
}

// greatest common divisor
pub fn gcd(mut a: i64, mut b: i64) -> i64 {
    while b != 0 {
        (a, b) = (b, a % b);
    }
    a.abs()
}

// least common multiple
pub fn lcm(a: i64, b: i64) -> i64 {
    a * b / gcd(a, b)
}

#[derive(Debug, Eq, PartialEq, Copy, Clone)]
pub enum Pulse {
    Low = 0,
    High
}

#[derive(Debug, Clone)]
pub struct Broadcaster {
    pub name: String,
    pub destinations: Vec<String>,
}

impl Broadcaster {
    pub fn new(destinations: Vec<String>) -> Self {
        Self { 
            name: "broadcaster".into(),
            destinations
        }
    }

    pub fn send_pulse<'a, F>(&'a mut self, _source: &String, pulse: Pulse, trigger: &mut F)
        where F: FnMut(&'a String, Pulse) -> ()
    {
        for d in &self.destinations {
            trigger(d, pulse);
        }
    }
}

#[derive(Debug, Clone)]
pub struct FlipFlop {
    pub name: String,
    pub destinations: Vec<String>,
    pub on: bool,
}

impl FlipFlop {
    pub fn new(name: impl Into<String>, destinations: Vec<String>) -> Self {
        Self { 
            name: name.into(),
            on: false,
            destinations
        }
    }

    /*
        Flip-flop modules (prefix %) are either on or off; they are initially off.
        If a flip-flop module receives a high pulse, it is ignored and nothing happens.
        However, if a flip-flop module receives a low pulse, it flips between on and
        off. If it was off, it turns on and sends a high pulse. If it was on, it turns
        off and sends a low pulse.
    */
    pub fn send_pulse<'a, F>(&'a mut self, _source: &String, pulse: Pulse, trigger: &mut F)
        where F: FnMut(&'a String, Pulse) -> ()
    {
        if let Pulse::Low = pulse {
            self.on = !self.on;
            let p = if self.on { Pulse::High } else { Pulse::Low };

            for d in &self.destinations {
                trigger(d, p);
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct Conjunction {
    pub name: String,
    pub destinations: Vec<String>,
    pub inputs: HashMap<String, Pulse>,
}

impl Conjunction {
    pub fn new(name: impl Into<String>, destinations: Vec<String>) -> Self {
        Self { 
            name: name.into(),
            destinations,
            inputs: HashMap::new(),
        }
    }

    /*
      Conjunction modules (prefix &) remember the type of the most recent pulse received
      from each of their connected input modules; they initially default to remembering 
      a low pulse for each input. When a pulse is received, the conjunction module first
      updates its memory for that input. Then, if it remembers high pulses for all inputs,
      it sends a low pulse; otherwise, it sends a high pulse.
    */
    pub fn send_pulse<'a, F>(&'a mut self, source: &String, pulse: Pulse, trigger: &mut F)
        where F: FnMut(&'a String, Pulse) -> ()
    {
        // Update the memory if it is different for the input.
        match self.inputs.get(source) {
            Some(p) if *p != pulse => {
                self.inputs.insert(source.clone(), pulse);
            },
            None => {
                self.inputs.insert(source.clone(), pulse);
            },
            _ => {}
        }
    
        // Which pulse should be sent.
        let pulse_to_send =
            if self.inputs.values().all(|p| *p == Pulse::High) {
                Pulse::Low
            }
            else {
                Pulse::High
            };


        // Send the pulse through
        for d in &self.destinations {
            trigger(d, pulse_to_send);
        }
    }

    pub fn connect(&mut self, input: &String) {
        self.inputs.insert(input.clone(), Pulse::Low);
    }
}

#[derive(Debug, Clone)]
pub enum Module {
    BroadcasterType(Broadcaster),
    FlipFlopType(FlipFlop),
    ConjunctionType(Conjunction),
}

impl Module {

    pub fn send_pulse<'a, F>(&'a mut self, source: &String, pulse: Pulse, trigger: &mut F)
        where F: FnMut(&'a String, Pulse) -> ()
    {
        match self {
            Self::BroadcasterType(b) => b.send_pulse(source, pulse, trigger),
            Self::FlipFlopType(ff) => ff.send_pulse(source, pulse, trigger),
            Self::ConjunctionType(c) => c.send_pulse(source, pulse, trigger),
        }
    }

    pub fn get_name(&self) -> &String {
        match self {
            Self::BroadcasterType(b) => &b.name,
            Self::FlipFlopType(ff) => &ff.name,
            Self::ConjunctionType(c) => &c.name,
        }
    }
    
    pub fn get_destinations(&self) -> &Vec<String> {
        match self {
            Self::BroadcasterType(b) => &b.destinations,
            Self::FlipFlopType(ff) => &ff.destinations,
            Self::ConjunctionType(c) => &c.destinations,
        }
    }

    pub fn connect(&mut self, input: &String) {
        match self {
            Self::ConjunctionType(c) => c.connect(input),
            _ => {}
        }
    }

    pub fn parse(text: impl AsRef<str>) -> AOCResult<Module> {
        let text = text.as_ref().trim_end();

        let cap = MODULE_REGEX.captures_must(text.as_ref())?;

        let module_name = cap.get_group(2)?;

        let destinations: Vec<String> = cap
            .get_group(3)?
            .split(',')
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string())
            .collect();

        if let Some(module_type_grp) = cap.get(1) {
            Ok(match module_type_grp.as_str() {
                "%" => {
                    Module::FlipFlopType(FlipFlop::new(module_name, destinations))
                },
                "&" => {
                    Module::ConjunctionType(Conjunction::new(module_name, destinations))
                },
                _ => {
                    return Err(AOCError::ParseError(format!("Invalid module line: {}", text)))
                }
            })
        }
        else if module_name == "broadcaster" {
            Ok(Module::BroadcasterType(Broadcaster::new(destinations)))
        }
        else {
            return Err(AOCError::ParseError(format!("Invalid module line: {}", text)))
        }
    }
}

#[derive(Debug, Clone)]
pub struct Modules {
    pub modules: HashMap<String, Module>,
}

impl Modules {

    pub fn new() -> Self {
        Self { modules: HashMap::new() }
    }

    pub fn add(&mut self, module: Module) {
        self.modules.insert(module.get_name().clone(), module);
    }

    // Initiates connections between modules.
    // This informs them of their inputs.
    pub fn connect(&mut self) -> AOCResult<()> {
        let mut connections: Vec<(String, String)> = Vec::new();

        // Tell modules about their connected inputs.
        // They already know their outputs.
        for (_, m) in self.modules.iter() {
            for d in m.get_destinations() {
                connections.push((m.get_name().clone(), d.clone()));
            }
        }

        for (source, destination) in connections {
            match self.modules.get_mut(&destination) {
                None => {
                    // I think this should have been an error.
                },
                Some(m) => {
                    m.connect(&source);
                }
            }
        }

        Ok(())
    }

    pub fn parse(input: impl AsRef<Path>) -> AOCResult<Modules> {
        let reader = BufReader::new(File::open(input)?);
        let mut modules = Modules::new();

        for line in reader.lines() {
            let line = line?;
            modules.add(Module::parse(line)?);
        }

        modules.connect()?;

        Ok(modules)
    }

    fn find_rx_input(&self) -> AOCResult<Conjunction> {
        let rx_name = String::from("rx");

        for m in self.modules.values() {
            if m.get_destinations().contains(&rx_name) {
                match m {
                    Module::ConjunctionType(c) => {
                        return Ok(c.clone());
                    },
                    _ => {}
                }
            }
        }

        Err(AOCError::ProcessingError(format!("Not able to find the expected input type.")))
    }

    pub fn find_button_pushes_into_rx_single_low(&mut self) -> AOCResult<i64> {

        // This is such a hack and works based on some assumptions about the data.
        // Assumption 1: the input to rx is a Conjunction node.
        // Assumption 2: there is a pattern to the cycles of each input coming into
        //                the conjunction node. We can use these cycles to figure out
        //                when they match.


        // Start by finding the input to rx and creating a map of the rx inputs inputs.
        // When all the hash maps have found the first high.
        let rx_input = self.find_rx_input()?;

        let mut input_trigger_counts: HashMap<String, Option<i32>> = HashMap::new();

        for conjunction_input_name in rx_input.inputs.keys() {
            input_trigger_counts.insert(conjunction_input_name.clone(), None);
        }

        // Keep trigging the button until we see all the highs for the rx inputs inputs.
        let broadcaster = String::from("broadcaster");
        let mut button_push_count = 0;

        while input_trigger_counts.values().any(|c| c.is_none()) {
            button_push_count += 1;

            // Send the button push through and see if Highs are hit for the conjunction.
            self.send_pulse(broadcaster.clone(), Pulse::Low, &mut |_source, _dst, dst_module, _pulse| {
                match dst_module {
                    Some(Module::ConjunctionType(dst_module)) => {
                        if dst_module.name == rx_input.name {
                            for (input_name, last_pulse) in &dst_module.inputs {
                                if last_pulse == &Pulse::High {
                                    input_trigger_counts.insert(input_name.clone(), Some(button_push_count));
                                }
                            }
                        }
                    },
                    _ => {}
                }
            })?;
        }

        let common_cycle = input_trigger_counts
            .values()
            .filter_map(|x| *x)
            .map(|x| x as i64)
            .reduce(|a, b| lcm(a, b))
            .ok_or_else(|| AOCError::ProcessingError("Couldn't calculate cycle".into()))?;

        return Ok(common_cycle);
    }

    pub fn push_button(&mut self, n: i32)-> AOCResult<(i32, i32)> {
        let broadcaster = String::from("broadcaster");
        let mut high_pulse_count = 0;
        let mut low_pulse_count = 0;

        for _push_count in 0 .. n {
            self.send_pulse(broadcaster.clone(), Pulse::Low, &mut |_source, _destination, _destination_module, pulse| {
                match pulse {
                    Pulse::High => high_pulse_count += 1,
                    Pulse::Low => low_pulse_count += 1,
                }
            })?;
        }

        Ok((high_pulse_count, low_pulse_count))
    }

    pub fn send_pulse<F>(&mut self, name: String, pulse: Pulse, on_pulse: &mut F) -> AOCResult<()>
        where F: FnMut(&String, &String, Option<&Module>, Pulse) -> ()
    {
        let initial = String::from("button");

        let mut pulses_to_send: VecDeque<(String, String, Pulse)> = VecDeque::new();
        pulses_to_send.push_back((initial, name, pulse));

        while let Some((source, destination, pulse)) = pulses_to_send.pop_front() {
            match self.modules.get_mut(&destination) {
                None => {
                    // missing module is a sink
                    on_pulse(&source, &destination, None, pulse);
                },
                Some(m) => {
                    m.send_pulse(&source, pulse, &mut |trigger, trigger_pulse| {
                        pulses_to_send.push_back((destination.clone(), trigger.clone(), trigger_pulse))
                    });
                    on_pulse(&source, &destination, Some(m), pulse);
                }
            }
        }
        Ok(())
    }
}

pub fn part1(input: impl AsRef<Path>) -> AOCResult<String> {
    let mut modules = Modules::parse(input)?;
    let (high_pulse_count, low_pulse_count) = modules.push_button(1000)?;

    let result = high_pulse_count * low_pulse_count;
    Ok(result.to_string())
}

pub fn part2(input: impl AsRef<Path>) -> AOCResult<String> {
    let mut modules = Modules::parse(input)?;
    let result = modules.find_button_pushes_into_rx_single_low()?;
    Ok(result.to_string())
}

