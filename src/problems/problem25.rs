use std::collections::HashMap;
use std::collections::HashSet;

use std::fs::File;
use std::io::BufReader;
use std::io::prelude::*;
use std::path::Path;

use rand::Rng;
use rand;

use crate::aocbase::{AOCResult, AOCError};

#[derive(Debug, Clone)]
pub struct ComponentGraph {
    pub edges: HashMap<String, HashSet<String>>,
}

impl ComponentGraph {
    
    pub fn new() -> Self {
        Self { edges: HashMap::new() }
    }

    pub fn load(input_file: impl AsRef<Path>) -> AOCResult<Self> {
        let reader = BufReader::new(File::open(input_file.as_ref())?);
        let mut graph = ComponentGraph::new();

        for line in reader.lines() {
            let line = line?;
            let line = line.trim();

            let mut top_parts = line.split(':');

            let node = top_parts
                .next()
                .ok_or_else(|| AOCError::ParseError(format!("Invalid line: {}", line)))?;

            let remaining = top_parts
                .next()
                .ok_or_else(|| AOCError::ParseError(format!("Invalid line: {}", line)))?;

            for connected_node in remaining.split_ascii_whitespace() {
                if connected_node.len() > 0 {
                    graph.add(node, connected_node);
                }
            }
        }

        Ok(graph)
    }

    pub fn add(&mut self, node1: impl AsRef<str>, node2: impl AsRef<str>) {
        self._add_direction(node1.as_ref(), node2.as_ref());
        self._add_direction(node2.as_ref(), node1.as_ref());
    }

    pub fn _add_direction(&mut self, node1: &str, node2: &str) {
        match self.edges.get_mut(node1) {
            None => {
                let mut node_set = HashSet::<String>::new();
                node_set.insert(node2.to_string());
                self.edges.insert(node1.to_string(), node_set);
            },
            Some(node_set) => {
                node_set.insert(node2.to_string());
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct KCSNode<'a> {
    id: i32,
    nodes: HashSet<&'a String>,
    connections: HashMap<i32, Vec<(&'a String, &'a String)>>,
}

impl<'a> KCSNode<'a> {

    pub fn new(id: i32) -> Self {
        Self { id, nodes: HashSet::new(), connections: HashMap::new() }
    }

    pub fn add_node(&mut self, node: &'a String) {
        self.nodes.insert(node);
    }
}

pub struct KargersCutSolver<'a> {
    pub graph: &'a ComponentGraph,

    // Maps nodes from name to node id of algorithm
    pub node_map: HashMap<&'a String, i32>,

    pub sgraph_edges: HashMap<i32, KCSNode<'a>>,
}

impl<'a> KargersCutSolver<'a> {

    pub fn new(graph: &'a ComponentGraph) -> Self {
        Self {
            graph,
            node_map: HashMap::new(),
            sgraph_edges: HashMap::new()
        }
    }

    #[allow(dead_code)]
    pub fn pretty_print(&self) -> String {
        let mut out = String::new();

        out.push_str(format!("Node Map:\n").as_str());
        for (name, id) in &self.node_map {
            out.push_str(format!("  * {} -> {}\n", name, id).as_str());
        }

        out.push_str(format!("Super Nodes:\n").as_str());

        let mut ids = self.sgraph_edges.keys().map(|id| *id).collect::<Vec<i32>>();
        ids.sort();

        for id in ids {
            let node = &self.sgraph_edges[&id];
            out.push_str(format!("  SNode: {}\n", id).as_str());
            out.push_str(format!("    Contains:\n").as_str());
            for contained in &node.nodes {
                out.push_str(format!("      + {}\n", contained).as_str());
            }
            out.push_str(format!("    Connections:\n").as_str());
            for (connected_id, original_edges) in &node.connections {
                out.push_str(format!("      * {} -> edge_count: {}\n", connected_id, original_edges.len()).as_str());
            }
        }

        out
    }

    pub fn get_edge_product(&self) -> i32 {
        self.sgraph_edges.values().map(|node| node.nodes.len()).product::<usize>() as i32
    }

    pub fn solve(&mut self, target_min_cut: i32, max_iterations: i32) -> AOCResult<i32> {

        for iteration in 0 .. max_iterations {
            self.initialize_condensed_graph();
            self.condense()?;
    
            let node = self.sgraph_edges.values().nth(0).unwrap();
            let min_cut = node.connections.values().nth(0).unwrap().len() as i32;

            if min_cut <= target_min_cut {
                return Ok(iteration + 1);
            }
        }

        Err(AOCError::ProcessingError("Could not determine min cut.".into()))
    }

    fn condense(&mut self) -> AOCResult<()> {
        while self.sgraph_edges.len() > 2 {
            self.condense_one()?;
        }

        Ok(())
    }

    fn condense_one(&mut self) -> AOCResult<()> {
        let (node_id1, node_id2) = self
            .pick_random_edge()
            .ok_or_else(|| AOCError::ProcessingError(format!("Not enough edges.")))?;

        let mut node1 = self.sgraph_edges.remove(&node_id1).unwrap();
        let node2 = self.sgraph_edges.remove(&node_id2).unwrap();

        // Update node map.
        for original_node_id in &node2.nodes {
            self.node_map.insert(original_node_id, node1.id);
        }

        // Update the nodes pointing to the node being consumed.
        for (incoming_node_id, _) in &node2.connections {
            if let Some(mut incoming_node) = self.sgraph_edges.remove(incoming_node_id) {
                match incoming_node.connections.remove(&node2.id) {
                    None => {},
                    Some(original_edges) => {
                        match incoming_node.connections.remove(&node1.id) {
                            None => {
                                incoming_node.connections.insert(node1.id, original_edges);
                            },
                            Some(mut node1_edges) => {
                                for o_edge in original_edges {
                                    node1_edges.push(o_edge);
                                }
                                incoming_node.connections.insert(node1.id, node1_edges);
                            }
                        }

                    }
                }
                self.sgraph_edges.insert(*incoming_node_id, incoming_node);
            }
        }

        // Add original node contained in 2 to 1.
        for contained_id in &node2.nodes {
            node1.nodes.insert(contained_id);
        }

        // Add edges from 2 to 1.
        for (connected_node_id, original_edges) in node2.connections {
            let mut node1_original_edges = match node1.connections.remove(&connected_node_id) {
                Some(prev_n1_edges) => prev_n1_edges,
                None => Vec::new(),
            };
            for o_edge in original_edges {
                node1_original_edges.push(o_edge);
            }
            node1.connections.insert(connected_node_id, node1_original_edges);
        }

        // Remove self loops.
        node1.connections.remove(&node1.id);
        node1.connections.remove(&node2.id);
        
        // Add node 1 back
        self.sgraph_edges.insert(node1.id, node1);

        Ok(())
    }

    fn pick_random_edge(&self) -> Option<(i32, i32)> {
        let mut rng = rand::thread_rng();
        let mut chosen = Option::<(f32, i32, i32)>::None;

        // Linear scan to pick 1 edge at random.

        for (id, node) in &self.sgraph_edges {
            for (o_id, o_edges) in &node.connections {
                for _ in o_edges {
                    let rv = rng.gen::<f32>();
                    let is_chosen = match chosen {
                        None => true,
                        Some((prev_rv, _, _)) if rv > prev_rv => true,
                        _ => false
                    };
                    if is_chosen {
                        chosen = Some((rv, *id, *o_id));
                    }
                }
            }
        }

        chosen.map(|t| (t.1, t.2))
    }

    fn initialize_condensed_graph(&mut self) {

        // Clear previous state.
        self.sgraph_edges.clear();

        // Create initial super nodes
        for (k, _) in &self.graph.edges {
            let node_id = self.sgraph_edges.len() as i32;
            let mut node = KCSNode::new(node_id);
            node.add_node(k);
            self.sgraph_edges.insert(node_id, node);
            self.node_map.insert(k, node_id);
        }

        // Connect the super nodes
        for (_, node) in self.sgraph_edges.iter_mut() {
            let original_node_name = *node.nodes.iter().nth(0).unwrap();
            for original_connected_name in &self.graph.edges[original_node_name] {
                let connected_id = self.node_map[original_connected_name];
                node.connections.insert(connected_id, vec![(original_node_name, original_connected_name)]);
            }
        }
    }
}

pub fn part1(input: impl AsRef<Path>) -> AOCResult<String> {
    let graph = ComponentGraph::load(input)?;
    let mut solver = KargersCutSolver::new(&graph);

    let iteration_count = solver.solve(3, 1000)?;
    println!("Took {} iterations to find result.", iteration_count);
    //println!("Graph: {}", solver.pretty_print());

    let result = solver.get_edge_product();
    Ok(result.to_string())
}