use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;
use std::io::prelude::*;
use std::path::Path;

use lazy_static::lazy_static;
use regex::Regex;

use crate::aocbase::{AOCResult, AOCError};

lazy_static! {
    static ref WORKFLOW_REGEX: Regex = Regex::new(
        r"^\s*([a-zA-Z]+)\{([^\}]*)\}\s*$"
    ).unwrap();
    
    static ref PART_REGEX: Regex = Regex::new(
        r"^\s*\{([^\}]+)\}\s*$"
    ).unwrap();

    static ref STEP_REGEX: Regex = Regex::new(
        r"^\s*(([xmas])([<>])(\d+):)?([a-zA-Z]+)\s*$"
    ).unwrap();
}


/*
    x: Extremely cool looking
    m: Musical (it makes a noise when you hit it)
    a: Aerodynamic
    s: Shiny
*/

#[derive(Debug, PartialEq, Eq, Copy, Clone, Hash)]
pub enum PartAttribute {
    Cool = 0,
    Musical,
    Aerodynamic,
    Shiny,
}

impl PartAttribute {
    pub fn from_char(c: char) -> AOCResult<PartAttribute> {
        use PartAttribute::*;
        Ok(match c {
            'x' => Cool,
            'm' => Musical,
            'a' => Aerodynamic,
            's' => Shiny,
            _ => return Err(AOCError::ParseError(format!("Invalid part attribute: {c}")))
        })
    }
}

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub struct Part {
    pub cool: i32,
    pub musical: i32,
    pub aerodynamic: i32,
    pub shiny: i32,
}

impl Part {

    pub fn rating(&self) -> i32 {
        self.cool + self.musical + self.aerodynamic + self.shiny
    }

    pub fn get_attribute(&self, attr: &PartAttribute) -> i32 {
        use PartAttribute::*;

        match attr {
            Cool => self.cool,
            Musical => self.musical,
            Aerodynamic => self.aerodynamic,
            Shiny => self.shiny,
        }
    }

    pub fn parse(line: impl AsRef<str>) -> AOCResult<Self> {
        use PartAttribute::*;

        let line = line.as_ref();

        let attr_parts = PART_REGEX
            .captures(line)
            .ok_or_else(|| AOCError::ParseError(format!("Invalid part: {}", line)))?
            .get(1)
            .ok_or_else(|| AOCError::InvalidRegexOperation("Invalid regex capture(1)".into()))?
            .as_str()
            .split(',');

        let mut attrs: HashMap<PartAttribute, i32> = HashMap::new();

        for attr in attr_parts {
            let s_parts: Vec<&str> = attr.split('=').collect();
            if s_parts.len() != 2 {
                return Err(AOCError::ParseError(format!("Invalid part attribute: {}", attr)));
            }

            let attr_type = PartAttribute::from_char(s_parts[0].chars().nth(0).unwrap())?;
            let attr_num = s_parts[1].parse::<i32>()?;

            attrs.insert(attr_type, attr_num);
        }

        for attr_type in [Cool, Musical, Aerodynamic, Shiny] {
            if !attrs.contains_key(&attr_type) {
                return Err(AOCError::ParseError(format!("Missing attribute: {:?}", attr_type)));
            }
        }

        Ok(Part {
            cool: attrs[&Cool],
            musical: attrs[&Musical],
            aerodynamic: attrs[&Aerodynamic],
            shiny: attrs[&Shiny],
        })
    }
}

#[derive(Debug, Clone)]
pub enum WorkflowStepCondition {
    LessThan(PartAttribute, i32),
    GreaterThan(PartAttribute, i32),
    True,
}

impl WorkflowStepCondition {
    pub fn matches(&self, part: &Part) -> bool {
        use WorkflowStepCondition::*;

        match self {
            LessThan(attr, num) => part.get_attribute(attr) < *num,
            GreaterThan(attr, num) => part.get_attribute(attr) > *num,
            True => true,
        }
    }
}

#[derive(Debug, Clone)]
pub enum WorkflowResult {
    Accept,
    Reject,
    Proceed(String),
}

#[derive(Debug, Clone)]
pub struct WorkflowStep {
    condition: WorkflowStepCondition,
    result: WorkflowResult,
}

impl WorkflowStep {

    pub fn process(&self, part: &Part) -> Option<WorkflowResult> {
        if self.condition.matches(part) {
            Some(self.result.clone())
        }
        else {
            None
        }
    }

    pub fn parse(text: impl AsRef<str>) -> AOCResult<Self> {
        let text = text.as_ref();

        let cap = STEP_REGEX
            .captures(text)
            .ok_or_else(|| AOCError::ParseError(format!("Invalid workflow step: {}", text)))?;

        // Parse the condition
        let condition =
            if let Some(part_attribute_group) = cap.get(2) {

                let part_attribute = PartAttribute::from_char(part_attribute_group
                    .as_str()
                    .chars()
                    .nth(0).unwrap()
                )?;

                let operation = cap
                    .get(3)
                    .ok_or_else(|| AOCError::ParseError("Invalid capture group(3)".into()))?
                    .as_str();

                let op_num = cap
                    .get(4)
                    .ok_or_else(|| AOCError::ParseError("Invalid capture group(4)".into()))?
                    .as_str()
                    .parse::<i32>()?;

                match operation {
                    "<" => WorkflowStepCondition::LessThan(part_attribute, op_num),
                    ">" => WorkflowStepCondition::GreaterThan(part_attribute, op_num),
                    _ => return Err(AOCError::ParseError(format!("Invalid operation in step condition.")))
                }
            }
            else {
                WorkflowStepCondition::True
            };

        // Get the target
        let target = cap
            .get(5)
            .ok_or_else(|| AOCError::ParseError("Invalid capture group(5)".into()))?
            .as_str();

        let result = match target {
            "A" => WorkflowResult::Accept,
            "R" => WorkflowResult::Reject,
            _ => WorkflowResult::Proceed(target.to_string()),
        };

        Ok(WorkflowStep { condition, result })
    }
}

#[derive(Debug, Clone)]
pub struct Workflow {
    pub name: String,
    pub steps: Vec<WorkflowStep>,
}

impl Workflow {

    pub fn process(&self, part: &Part) -> AOCResult<WorkflowResult> {
        for step in &self.steps {
            match step.process(part) {
                Some(result) => {
                    return Ok(result);
                },
                _ => {}
            }
        }

        Err(AOCError::ProcessingError(format!("Unable to process part: {:?}", part)))
    }

    pub fn parse(line: impl AsRef<str>) -> AOCResult<Self> {
        let line = line.as_ref();

        let cap = WORKFLOW_REGEX
            .captures(line)
            .ok_or_else(|| AOCError::ParseError(format!("Invalid workflow line: {}", line)))?;

        let name = cap
            .get(1)
            .ok_or_else(|| AOCError::InvalidRegexOperation("Invalid capture group (1)".into()))?
            .as_str()
            .to_string();

        let steps = cap
            .get(2)
            .ok_or_else(|| AOCError::InvalidRegexOperation("Invalid capture group (2)".into()))?
            .as_str()
            .split(',')
            .map(WorkflowStep::parse)
            .collect::<AOCResult<Vec<WorkflowStep>>>()?;

        Ok(Workflow { name, steps })
    }
}

#[derive(Debug, Clone)]
pub struct Workflows {
    pub workflows: HashMap<String, Workflow>,
}

impl Workflows {
    pub fn new() -> Self {
        Self { workflows: HashMap::new() }
    }

    pub fn add(&mut self, workflow: Workflow) {
        self.workflows.insert(workflow.name.clone(), workflow);
    }

    pub fn get_workflow<'a>(&'a self, name: impl AsRef<str>) -> AOCResult<&'a Workflow> {
        Ok(self.workflows.get(name.as_ref())
            .ok_or_else(|| AOCError::ProcessingError(format!("Missing workflow: {}", name.as_ref())))?)
    }

    pub fn process(&self, part: &Part) -> AOCResult<WorkflowResult> {
        use WorkflowResult::*;

        let mut work_flow = self.get_workflow("in")?;
        loop {
            let result = work_flow.process(part)?;
            match result {
                Accept|Reject => {
                    return Ok(result);
                },
                Proceed(next_workflow_name) => {
                    work_flow = self.get_workflow(next_workflow_name)?;
                }
            }
        }
    }
}

pub fn parse_worksheet(input: impl AsRef<Path>) -> AOCResult<(Workflows, Vec<Part>)> {
    let reader = BufReader::new(File::open(input)?);

    let mut workflows = Workflows::new();
    let mut parts: Vec<Part> = Vec::new();

    let mut in_workflows = true;

    for line in reader.lines() {
        let line = line?;
        let line = line.trim();

        if in_workflows {

            // blank line goes to next section
            if line.len() == 0 {
                in_workflows = false;
                continue;
            }

            workflows.add(Workflow::parse(line)?);
        }
        else if line.len() > 0 {
            parts.push(Part::parse(line)?);
        }
    }

    Ok((workflows, parts))
}

pub fn part1(input: impl AsRef<Path>) -> AOCResult<String> {
    let (workflows, parts) = parse_worksheet(input)?;

    let mut total_ratings = 0;

    for part in &parts {
        let part_result = workflows.process(part)?;
        if let WorkflowResult::Accept = part_result {
            total_ratings += part.rating();
        }
    }

    Ok(total_ratings.to_string())
}