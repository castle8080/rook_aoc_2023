use std::collections::HashMap;
use std::collections::HashSet;
use std::path::Path;

use lazy_static::lazy_static;
use regex::Regex;

use crate::aocbase::{AOCError, AOCResult};
use crate::aocio::each_line;
use crate::regex_ext::CapturesExt;

lazy_static! {
    static ref COMMAND_REGEX: Regex = Regex::new(r"^\s*([RL]+)\s*$").unwrap();
    static ref NODE_REGEX: Regex = Regex::new(r"^([A-Z0-9]{3}) = \(([A-Z0-9]{3}), ([A-Z0-9]{3})\)").unwrap();
}

#[derive(Debug)]
pub enum Command {
    Left,
    Right,
}

impl Command {
    pub fn parse(c: char) -> AOCResult<Command> {
        Ok(match c {
            'R' => Command::Right,
            'L' => Command::Left,
            _ => {
                return Err(AOCError::ParseError(format!("Invalid command: {c}")))
            }
        })
    }
}

#[derive(Debug)]
pub struct Node {
    id: String,
    left: String,
    right: String,
}

#[derive(Debug)]
pub struct Network {
    pub commands: Vec<Command>,
    pub nodes: HashMap<String, Node>,
}

impl Network {
    pub fn new() -> Self {
        Network { commands: Vec::new(), nodes: HashMap::new() }
    }

    pub fn get_node<'a>(&'a self, id: impl AsRef<str>) -> AOCResult<&'a Node> {
        Ok(self
            .nodes
            .get(id.as_ref())
            .ok_or_else(|| AOCError::ProcessingError(format!("Invalid start location: {}", id.as_ref())))?)
    }

    /// Gives a list of ids in order of encounter using commands and the visit step at which a cycle would start.
    pub fn search_cycle<'a>(&'a self, start: &str, commands: &Vec<Command>) -> AOCResult<(usize, Vec<&str>)> {
        let mut places: Vec<(usize, &str)> = Vec::new();
        let mut visited: HashSet<(usize, &str)> = HashSet::new();
        let mut node = self.get_node(start)?;

        for (i, c) in commands.iter().enumerate().cycle() {
            let pos = (i, node.id.as_str());

            if visited.contains(&pos) {
                let cycle_start = places.iter().position(|place| *place == pos).unwrap();
                return Ok((cycle_start, places.iter().map(|place| place.1).collect::<Vec<&str>>()))
            }
            else {
                places.push(pos.clone());
                visited.insert(pos);
            }

            let next_id = match c {
                Command::Left => &node.left,
                Command::Right => &node.right,
            };

            node = self.get_node(next_id)?;
        }

        return Err(AOCError::ProcessingError("never!".into()))
    }

    pub fn search(&self, start: &str, end: &str, commands: &Vec<Command>) -> AOCResult<i32> {
        let mut steps = 0;

        let mut cur_node = self
            .nodes
            .get(start)
            .ok_or_else(|| AOCError::ProcessingError(format!("Invalid start location: {start}")))?;

        for c in commands.iter().cycle() {
            if cur_node.id == end {
                return Ok(steps);
            }

            let next_id = match c {
                Command::Left => &cur_node.left,
                Command::Right => &cur_node.right,
            };

            cur_node = self
                .nodes
                .get(next_id)
                .ok_or_else(|| AOCError::ProcessingError(format!("Invalid location: {next_id}")))?;

            steps += 1;
        }

        Err(AOCError::ProcessingError(format!("Could not find end: {end}")))
    }

    pub fn add_node(&mut self, node: Node) {
        self.nodes.insert(node.id.clone(), node);
    }

    pub fn parse(input: impl AsRef<Path>) -> AOCResult<Self> {
        let mut network = Network::new();

        each_line(input, |line| {
            if let Some(command_cap)  = COMMAND_REGEX.captures(line) {
                let commands = command_cap
                    .get_group(1)?
                    .chars()
                    .map(Command::parse)
                    .collect::<AOCResult<Vec<Command>>>()?;

                network.commands = commands;
            }
            else if let Some(node_cap) = NODE_REGEX.captures(line) {
                let id = node_cap
                    .get_group(1)?
                    .to_string();

                let left = node_cap
                    .get_group(2)?
                    .to_string();

                let right = node_cap
                    .get_group(3)?
                    .to_string();

                network.add_node(Node { id, left, right })
            }
            else if line.trim_end().len() > 0 {
                return Err(AOCError::ParseError(format!("Invalid line: {line}")));
            }
            Ok(())
        })?;

        Ok(network)
    }
}

pub fn part1(input: impl AsRef<Path>) -> AOCResult<String> {
    let network = Network::parse(input)?;
    let result = network.search("AAA", "ZZZ", &network.commands)?;

    Ok(result.to_string())
}

/// Using information about a cycle in the network and choosing target nodes
/// of interest you can figure out when the next target node will be visited.
#[derive(Debug)]
pub struct NetworkCycleIterator {
    pub pre_cycle: Vec<usize>,
    pub in_cycle: Vec<usize>,
    pub cycle_start: usize,
    pub visit_length: usize,
}

impl NetworkCycleIterator {

    /// Creates a new cycle iterator.
    /// network: The network
    /// start: where the search in the network starts.
    /// end_func: Tells if a node id is of interest (potential end).
    pub fn new<F>(network: &Network, start: &str, target_func: F) -> AOCResult<NetworkCycleIterator>
        where F: Fn(&str) -> bool
    {
        let (cycle_start, ids) = network.search_cycle(start, &network.commands)?;

        let mut pre_cycle: Vec<usize> = Vec::new();
        let mut in_cycle: Vec<usize> = Vec::new();

        for (n, id) in ids.iter().enumerate() {
            if target_func(id) {
                if n < cycle_start {
                    pre_cycle.push(n);
                }
                else {
                    in_cycle.push(n);
                }
            }
        }

        Ok(NetworkCycleIterator {
            pre_cycle,
            in_cycle,
            cycle_start,
            visit_length: ids.len(),
        })
    }

    /// For the nth target node get the step
    pub fn get_step_for_nth_target(&self, nth: usize) -> usize {
        if nth < self.pre_cycle.len() {
            self.pre_cycle[nth]
        }
        else {
            let cycle_zth = nth - self.pre_cycle.len();
    
            let step = self.in_cycle[cycle_zth % self.in_cycle.len()] +
                (cycle_zth / self.in_cycle.len()) * (self.visit_length - self.cycle_start);

            step
        }
    }
}

#[derive(Debug)]
struct NCIterState {
    iterator: NetworkCycleIterator,
    nth: usize,
    step: usize,
}

impl NCIterState {
    pub fn new(iterator: NetworkCycleIterator) -> Self {
        let nth = 0;
        let step = iterator.get_step_for_nth_target(nth);
        NCIterState { iterator, nth, step }
    }

    pub fn next(&mut self) -> usize {
        self.nth += 1;
        self.step = self.iterator.get_step_for_nth_target(self.nth);
        self.step
    }
}

fn find_common_step(nc_iter_states: &mut Vec<NCIterState>) -> usize {
    let mut max_step = nc_iter_states[0].step;

    loop {
        let mut all_match = true;
        for st in nc_iter_states.iter_mut() {

            while st.step < max_step {
                st.next();
            }

            if st.step > max_step {
                all_match = false;
                max_step = st.step;
            }
        }
        if all_match {
            return max_step;
        }
    }
}

pub fn part2(input: impl AsRef<Path>) -> AOCResult<String> {
    let network = Network::parse(input)?;

    /*
        Finds a cycle in going through the commands for each start
        Using this cycle you can map out each ending node and instead of 
        walking each node, you skip steps using the cycle.
        This way you can iterate over each start looking at the next target item
        in step order and see when steps match.
        This is still slower than I would like, but I don't have another algorithm yet.
        Running ~ 15 seconds.
    */

    let starts = network.nodes
        .keys()
        .filter(|node| node.ends_with("A"))
        .map(|node| node.as_str())
        .collect::<Vec<&str>>();

    let mut nc_iter_states: Vec<NCIterState> = Vec::new();

    for start in starts {
        let iterator = NetworkCycleIterator::new(&network, start, |id| id.ends_with("Z"))?;
        nc_iter_states.push(NCIterState::new(iterator));
    }

    let result = find_common_step(&mut nc_iter_states);

    Ok(result.to_string())
}