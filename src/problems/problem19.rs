use std::collections::HashMap;
use std::collections::HashSet;
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

    // Splits up part combinations into those that match and those that don't.
    pub fn process_combinations(&self, part_combinations: &PartAttributeCombination)
        -> AOCResult<(WorkflowResult, PartAttributeCombination, PartAttributeCombination)>
    {
        use WorkflowStepCondition::*;

        Ok(match &self.condition {
            True => {
                (self.result.clone(), part_combinations.clone(), PartAttributeCombination::new_empty())
            },
            GreaterThan(attr, num) => {
                let (parts_in, parts_out): (HashSet<i32>, HashSet<i32>) = part_combinations
                    .get(&attr)
                    .iter()
                    .partition(|v| *v > &num);

                (self.result.clone(),
                    part_combinations.with_attributes(attr, parts_in),
                    part_combinations.with_attributes(attr, parts_out))
            },
            LessThan(attr, num) => {
                let (parts_in, parts_out): (HashSet<i32>, HashSet<i32>) = part_combinations
                    .get(&attr)
                    .iter()
                    .partition(|v| *v < &num);

                (self.result.clone(),
                    part_combinations.with_attributes(attr, parts_in),
                    part_combinations.with_attributes(attr, parts_out))
            },
        })
    }

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

    // Splits the part attribute combination into the different set of results it could have.
    pub fn process_combinations(&self, part_combinations: &PartAttributeCombination)
        -> AOCResult<Vec<(WorkflowResult, PartAttributeCombination)>>
    {
        let mut result: Vec<(WorkflowResult, PartAttributeCombination)> = Vec::new();
        self.process_combinations_recur(0, part_combinations, &mut result)?;
        Ok(result)
    }

    // Recursion helps me not clone combinations as much
    fn process_combinations_recur(&self,
        step_idx: usize,
        remaining_part_combinations: &PartAttributeCombination,
        result: &mut Vec<(WorkflowResult, PartAttributeCombination)>)
        -> AOCResult<()>
    {
        if step_idx >= self.steps.len() {
            return Ok(());
        }
        let step = &self.steps[step_idx];
        let (step_result, step_in, step_out) = step.process_combinations(&remaining_part_combinations)?;

        if !step_in.is_empty() {
            result.push((step_result.clone(), step_in));
        }

        if !step_out.is_empty() {
            self.process_combinations_recur(step_idx + 1, &step_out, result)?;
        }

        Ok(())
    }
    
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

    fn get_accepted_combinations_recur(
        &self,
        part_combinations: &PartAttributeCombination,
        name: impl AsRef<str>,
        result_combinations: &mut Vec<PartAttributeCombination>) -> AOCResult<()>
    {
        let workflow = self.get_workflow(name)?;

        for (wf_result, sub_part_combinations) in workflow.process_combinations(part_combinations)? {
            if !sub_part_combinations.is_empty() {
                match wf_result {
                    WorkflowResult::Accept => {
                        result_combinations.push(sub_part_combinations);
                    },
                    WorkflowResult::Reject => {
                        // skip
                    },
                    WorkflowResult::Proceed(next_wf_name) => {
                        self.get_accepted_combinations_recur(
                            &sub_part_combinations,
                            next_wf_name,
                            result_combinations)?;
                    }
                }
            }
        }

        Ok(())
    }

    pub fn get_accepted_combinations(&self, part_combinations: &PartAttributeCombination)
        -> AOCResult<Vec<PartAttributeCombination> >
    {
        let mut accepted_part_combos: Vec<PartAttributeCombination> = Vec::new();

        self.get_accepted_combinations_recur(
            part_combinations,
            "in",
            &mut accepted_part_combos
        )?;
        
        Ok(accepted_part_combos)
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

// I realize now that I could have based this completely off of range specs and not
// have to expand the whole HashSet. So it could have been 4 i32 pairs to represent
// the combos. This is because the conditions are only greater/less than operations.
#[derive(Debug, Clone)]
pub struct PartAttributeCombination {
    pub cool: HashSet<i32>,
    pub musical: HashSet<i32>,
    pub aerodynamic: HashSet<i32>,
    pub shiny: HashSet<i32>,
}

impl PartAttributeCombination {

    pub fn get_combination_size(&self) -> i64 {
        self.cool.len() as i64 *
            self.musical.len() as i64 *
            self.aerodynamic.len() as i64 *
            self.shiny.len() as i64
    }

    // If the combination is empty.
    pub fn is_empty(&self) -> bool {
        // If any set is empty the whole thing is empty.
        self.cool.is_empty() ||
            self.musical.is_empty() ||
            self.aerodynamic.is_empty() ||
            self.shiny.is_empty()
    }

    pub fn new_empty() -> Self {
        Self {
            cool: HashSet::new(),
            musical: HashSet::new(),
            aerodynamic: HashSet::new(),
            shiny: HashSet::new(),
        }
    }

    pub fn new(min: i32, max: i32) -> Self {
        let starting_vals: HashSet<i32> = (min ..= max).collect();
        Self {
            cool: starting_vals.clone(),
            musical: starting_vals.clone(),
            aerodynamic: starting_vals.clone(),
            shiny: starting_vals.clone(),
        }
    }

    pub fn with_attributes(&self, attr: &PartAttribute, vals: HashSet<i32>) -> Self {
        use PartAttribute::*;

        if vals.len() == 0 {
            Self {
                cool: HashSet::new(),
                musical: HashSet::new(),
                aerodynamic: HashSet::new(),
                shiny: HashSet::new(),
            }
        }
        else {
            // I want to move this to a macro
            match attr {
                Cool => Self {
                    cool: vals,
                    musical: self.musical.clone(),
                    aerodynamic: self.aerodynamic.clone(),
                    shiny: self.shiny.clone(),
                },
                Musical => Self {
                    cool: self.cool.clone(),
                    musical: vals,
                    aerodynamic: self.aerodynamic.clone(),
                    shiny: self.shiny.clone(),
                },
                Aerodynamic => Self {
                    cool: self.cool.clone(),
                    musical: self.musical.clone(),
                    aerodynamic: vals,
                    shiny: self.shiny.clone(),
                },
                Shiny => Self {
                    cool: self.cool.clone(),
                    musical: self.musical.clone(),
                    aerodynamic: self.aerodynamic.clone(),
                    shiny: vals,
                },
            }
        }
    }

    pub fn get<'a>(&'a self, attr: &PartAttribute) -> &'a HashSet<i32> {
        use PartAttribute::*;

        match attr {
            Cool => &self.cool,
            Musical => &self.musical,
            Aerodynamic => &self.aerodynamic,
            Shiny => &self.shiny,
        }
    }
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

pub fn part2(input: impl AsRef<Path>) -> AOCResult<String> {
    let (workflows, _parts) = parse_worksheet(input)?;

    let combinations = PartAttributeCombination::new(1, 4000);
    let accepted_combinations = workflows.get_accepted_combinations(&combinations)?;

    let mut total_combos: i64 = 0;

    for accepted_combination in &accepted_combinations {
        let size = accepted_combination.get_combination_size();
        total_combos += size;
    }

    Ok(total_combos.to_string())
}