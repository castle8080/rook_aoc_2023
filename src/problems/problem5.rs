use std::collections::HashMap;
use std::num::ParseIntError;
use std::path::Path;
use std::cmp;

use lazy_static::lazy_static;
use regex::Regex;

use crate::aocbase::{AOCError, AOCResult};
use crate::aocio::process_lines;

lazy_static! {
    static ref SEEDS_REGEX: Regex = Regex::new(r"^seeds: (.*)").unwrap();
    static ref MAP_START_REGEX: Regex = Regex::new(r"^([a-z]+)-to-([a-z]+) map:").unwrap();
}

fn update_min(opt: &mut Option<i64>, potential_min: i64) {
    match opt {
        None => *opt = Some(potential_min),
        Some(min) if potential_min < *min => *opt = Some(potential_min),
        _ =>{}
    }
}

#[derive(Debug, Clone)]
pub struct HorticultureRangeMap {
    pub destination_start: i64,
    pub source_start: i64,
    pub length: i64,
}

impl HorticultureRangeMap {
    pub fn translate(&self, n: i64) -> Option<i64> {
        if n >= self.source_start && n < self.source_start + self.length {
            Some(self.destination_start + (n - self.source_start))
        }
        else {
            None
        }
    }
}

#[derive(Debug, Clone)]
pub struct HorticultureMap {
    pub source_type: String,
    pub destination_type: String,
    pub range_maps: Vec<HorticultureRangeMap>,
}

pub struct SeedRangeMinTranslator<'a> {
    range_maps_sorted: Vec<&'a HorticultureRangeMap>,
}

impl<'a> SeedRangeMinTranslator<'a> {

    pub fn new(range_maps: &'a Vec<HorticultureRangeMap>) -> Self {
        let mut range_maps_sorted: Vec<&'a HorticultureRangeMap> = range_maps.iter().collect();
        range_maps_sorted.sort_by_key(|range_map| range_map.source_start);
        Self { range_maps_sorted }
    }

    pub fn translate(&self, start: i64, length: i64) -> Option<i64> {
        let end = start + length;

        let mut cur_passthrough_pos = start;
        let mut cur_min: Option<i64> = None;

        for range_map in self.range_maps_sorted.iter() {
            let overlap_start = cmp::max(start, range_map.source_start);
            let overlap_end = cmp::min(end, range_map.source_start + range_map.length);

            if overlap_start < overlap_end {
                // Check the overlap for new min
                let range_min_translation = range_map.translate(overlap_start).unwrap();
                update_min(&mut cur_min, range_min_translation);

                // check for gap jump
                if overlap_start > cur_passthrough_pos {
                    update_min(&mut cur_min, cur_passthrough_pos);
                    cur_passthrough_pos = overlap_end;
                }
            }
        }

        // Check for a remaining gap
        if cur_passthrough_pos < end {
            update_min(&mut cur_min, cur_passthrough_pos);
        }

        cur_min
    }
}

impl HorticultureMap {
    pub fn new(
        source_type: impl Into<String>,
        destination_type: impl Into<String>) -> Self
    {
        HorticultureMap {
            source_type: source_type.into(),
            destination_type: destination_type.into(),
            range_maps: Vec::new()
        }
    }

    pub fn add_range_map(&mut self, range_map: HorticultureRangeMap) {
        self.range_maps.push(range_map);
    }

    pub fn seed_range_min_translator<'a>(&'a self) -> SeedRangeMinTranslator<'a> {
        SeedRangeMinTranslator::new(&self.range_maps)
    }

    pub fn translate(&self, n: i64) -> i64 {
        for range_map in self.range_maps.iter() {
            if let Some(new_n) = range_map.translate(n) {
                return new_n;
            }
        }
        return n;
    }

    fn flatten_range_layer(
        cur_map_range: &HorticultureRangeMap,
        next_range_maps: &Vec<&HorticultureRangeMap>,
        new_range_maps: &mut Vec<HorticultureRangeMap>)
    {
        let cur_start = cur_map_range.source_start;
        let cur_end = cur_map_range.source_start + cur_map_range.length;
        let cur_delta = cur_map_range.destination_start - cur_map_range.source_start;
        let mut cur_pos = cur_start;

        for next_map_range in next_range_maps.iter() {
            let next_mapped_start = next_map_range.source_start - cur_delta;
            let next_mapped_end = cmp::min(next_mapped_start + next_map_range.length, cur_end);
            let next_delta = next_map_range.destination_start - next_map_range.source_start;

            if next_mapped_end < cur_pos {
                // not in range yet.
                continue;
            }
            else if next_mapped_start >= cur_end {
                // past need to look
                break;
            }
            else if next_mapped_start >= next_mapped_end {
                // skip invalid
                continue;
            }
            
            // There is a gap - only apply first delta
            if cur_pos < next_mapped_start {
                let new_map = HorticultureRangeMap {
                    source_start: cur_pos,
                    destination_start: cur_pos + cur_delta,
                    length: next_mapped_start - cur_pos,
                };
                new_range_maps.push(new_map);
                cur_pos = next_mapped_start;
            }

            // Overlap - apply both deltas
            if cur_pos < next_mapped_end {
                let new_map = HorticultureRangeMap {
                    source_start: cur_pos,
                    destination_start: cur_pos + cur_delta + next_delta,
                    length: next_mapped_end - cur_pos,
                };
                new_range_maps.push(new_map);
                cur_pos = next_mapped_end;
            }
        }

        // Check for last gap and apply first delta
        if cur_pos < cur_end {
            let new_map = HorticultureRangeMap {
                source_start: cur_pos,
                destination_start: cur_pos + cur_delta,
                length: cur_end - cur_pos,
            };
            new_range_maps.push(new_map);
        }
    }

    fn get_first_layer_hit_flattened_range_maps(
        cur_range_maps: &Vec<&HorticultureRangeMap>,
        next_range_maps: &Vec<&HorticultureRangeMap>,
        new_range_maps: &mut Vec<HorticultureRangeMap>)
    {
        for cur_map_range in cur_range_maps {
            HorticultureMap::flatten_range_layer(
                cur_map_range,
                next_range_maps,
                new_range_maps);
        }
    }

    fn get_first_layer_miss_to_second_layer_hit_maps(
        cur_range_maps: &Vec<&HorticultureRangeMap>,
        next_range_maps: &Vec<&HorticultureRangeMap>,
        new_range_maps: &mut Vec<HorticultureRangeMap>)
    {
        enum Layer { One, Two }
        enum EventType { Start, End }
        struct RangeEvent {
            id: i64,
            layer: Layer,
            event_type: EventType,
            delta: i64,
        }

        let mut range_events: Vec<RangeEvent> = Vec::new();

        for range_map in cur_range_maps {
            range_events.push(RangeEvent {
                id: range_map.source_start,
                layer: Layer::One,
                event_type: EventType::Start,
                delta: -1,
            });
            range_events.push(RangeEvent {
                id: range_map.source_start,
                layer: Layer::One,
                event_type: EventType::End,
                delta: -1,
            });
        }
        
        for range_map in next_range_maps {
            range_events.push(RangeEvent {
                id: range_map.source_start,
                layer: Layer::Two,
                event_type: EventType::Start,
                delta: range_map.destination_start - range_map.source_start,
            });
            range_events.push(RangeEvent {
                id: range_map.source_start,
                layer: Layer::Two,
                event_type: EventType::End,
                delta: range_map.destination_start - range_map.source_start,
            });
        }

        range_events.sort_by_key(|e| e.id);

        let mut in_layer_1 = false;
        let mut in_layer_2 = false;
        let mut delta: i64 = -1;
        let mut pos: i64 = -1;

        for e in &range_events {

            if pos >= 0 {
                // Something to process
                if !in_layer_1 && in_layer_2 {
                    let new_map = HorticultureRangeMap {
                        source_start: pos,
                        destination_start: pos + delta,
                        length: e.id - pos
                    };
                    new_range_maps.push(new_map);
                }
            }

            // Update state

            pos = e.id;

            match e {
                RangeEvent { layer: Layer::One, event_type: EventType::Start, .. } => {
                    in_layer_1 = true;
                },
                RangeEvent { layer: Layer::One, event_type: EventType::End, .. } => {
                    in_layer_1 = false;
                },
                RangeEvent { layer: Layer::Two, event_type: EventType::Start, .. } => {
                    in_layer_2 = true;
                    delta = e.delta;
                },
                RangeEvent { layer: Layer::Two, event_type: EventType::End, .. } => {
                    in_layer_2 = false;
                },
            }
        }
    }

    pub fn combine(&self, next_map: &HorticultureMap) -> HorticultureMap {
        let mut cur_range_maps: Vec<&HorticultureRangeMap> = self.range_maps.iter().collect();
        cur_range_maps.sort_by_key(|range_map| range_map.source_start);

        let mut next_range_maps: Vec<&HorticultureRangeMap> = next_map.range_maps.iter().collect();
        next_range_maps.sort_by_key(|range_map| range_map.source_start);

        let mut combined_range_maps: Vec<HorticultureRangeMap> = Vec::new();

        HorticultureMap::get_first_layer_hit_flattened_range_maps(
            &cur_range_maps,
            &next_range_maps,
            &mut combined_range_maps);

        HorticultureMap::get_first_layer_miss_to_second_layer_hit_maps(
            &cur_range_maps,
            &next_range_maps,
            &mut combined_range_maps);

        HorticultureMap {
            source_type: self.source_type.clone(),
            destination_type: next_map.destination_type.clone(),
            range_maps: combined_range_maps,
        }
    }
}

#[derive(Debug)]
pub struct HorticulturePlan {
    seeds: Vec<i64>,
    maps: HashMap<String, HorticultureMap>,
}

impl HorticulturePlan {

    pub fn new() -> Self {
        HorticulturePlan { seeds: Vec::new(), maps: HashMap::new() }
    }

    pub fn add_map(&mut self, map: HorticultureMap) {
        self.maps.insert(map.source_type.clone(), map);
    }

    pub fn get_seed_range_pairs(&self) -> Vec<(i64, i64)> {
        let end = self.seeds.len() / 2;
        (0 .. end)
            .map(|n| (self.seeds[n * 2], self.seeds[n*2 + 1]))
            .collect()
    }

    pub fn get_reduced(&self, starting: &str, ending: &str) -> Option<HorticultureMap> {
        match self.maps.get(starting) {
            None => None,
            Some(starting_map) => {
                let mut cur_map = starting_map.clone();
                if cur_map.destination_type == ending {
                    return Some(cur_map);
                }
                while let Some(next_map) = self.maps.get(cur_map.destination_type.as_str()) {
                    cur_map = cur_map.combine(next_map);
                    if cur_map.destination_type.as_str() == ending {
                        return Some(cur_map);
                    }
                }
                None
            }
        }
    }

    pub fn get_all_values<'a>(&'a self, seed: i64) -> HashMap<&'a str, i64> {
        let mut values_map: HashMap<&'a str, i64> = HashMap::new();
        let mut cur_mapping = self.maps.get("seed");
        let mut last_value = seed;

        if let Some(cv) = cur_mapping {
            values_map.insert(cv.source_type.as_str(), seed);
        }

        while let Some(cv) = cur_mapping {
            let next_value = cv.translate(last_value);

            values_map.insert(cv.destination_type.as_str(), next_value);

            last_value = next_value;
            cur_mapping = self.maps.get(cv.destination_type.as_str());
        }

        values_map
    }

    pub fn parse(input: impl AsRef<Path>) -> AOCResult<Self> {
        let mut plan = HorticulturePlan::new();
        let mut maps: Vec<HorticultureMap> = Vec::new();

        process_lines(input, |line| {
            let line = line.trim();

            // Skip blank lines
            if line.len() == 0 {
                // do nothing
            }
            else if let Some(seeds_cap) = SEEDS_REGEX.captures(line) {
                // Check for seeds line
                plan.seeds = seeds_cap
                    .get(1)
                    .ok_or_else(|| AOCError::InvalidRegexOperation("Invalid regex capture.".into()))?
                    .as_str()
                    .split_ascii_whitespace()
                    .map(|s| s.parse::<i64>())
                    .collect::<Result<Vec<i64>, ParseIntError>>()?;
            }
            else if let Some(map_start_cap) = MAP_START_REGEX.captures(line) {
                let source_type = map_start_cap
                    .get(1)
                    .ok_or_else(|| AOCError::InvalidRegexOperation("Invalid regex capture.".into()))?
                    .as_str();

                let destination_type = map_start_cap
                    .get(2)
                    .ok_or_else(|| AOCError::InvalidRegexOperation("Invalid regex capture.".into()))?
                    .as_str();

                maps.push(HorticultureMap::new(source_type, destination_type));
            }
            else {
                let map_range_numbers = line
                    .split_ascii_whitespace()
                    .map(|s| s.parse::<i64>())
                    .collect::<Result<Vec<i64>, ParseIntError>>()?;

                if map_range_numbers.len() != 3 {
                    return Err(AOCError::ParseError(format!("Invalid range mapping line: {}", line)));
                }

                // TODO: validate number ranges?
                let range_map = HorticultureRangeMap {
                    destination_start: map_range_numbers[0],
                    source_start: map_range_numbers[1],
                    length: map_range_numbers[2]
                };

                match maps.last_mut() {
                    None => {
                        return Err(AOCError::ParseError(format!("Unexpected line: {}", line)));
                    },
                    Some(map) => {
                        map.add_range_map(range_map);
                    }
                }
            }
            Ok(())
        })?;

        for map in maps {
            plan.add_map(map);
        }

        Ok(plan)
    }
}

pub fn part1(input: impl AsRef<Path>) -> AOCResult<String> {
    let plan = HorticulturePlan::parse(input)?;

    let mut location_min: Option<i64> = None;

    for seed in plan.seeds.iter() {
        let seed_values = plan.get_all_values(*seed);
        if let Some(loc) = seed_values.get("location") {
            update_min(&mut location_min, *loc);
        }
    }

    Ok(match location_min {
        None => "".into(),
        Some(min) => min.to_string()
    })
}

pub fn part2(input: impl AsRef<Path>) -> AOCResult<String> {
    let plan = HorticulturePlan::parse(input)?;

    let mut location_min: Option<i64> = None;

    if let Some(combined_map) = plan.get_reduced("seed", "location") {
        let seed_range_min_translator = combined_map.seed_range_min_translator();
        for (seed_start, seed_len) in plan.get_seed_range_pairs() {
            if let Some(min_trans) = seed_range_min_translator.translate(seed_start, seed_len) {
                update_min(&mut location_min, min_trans);
            }
        }
    };

    Ok(match location_min {
        None => "".into(),
        Some(min) => min.to_string()
    })
}
