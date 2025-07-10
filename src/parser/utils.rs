use std::path::PathBuf;

use crate::rules::{Rule, RulesList};
use shellexpand::tilde;

pub fn expand_path(path: &str) -> String {
    tilde(path).to_string()
}

pub fn handle_colon_end(mut path: String) -> String {
    if path.ends_with(':') {
        path += "\\";
    };
    path
}

pub fn process_path<S: AsRef<str>>(path: S) -> String {
    let p = expand_path(path.as_ref());
    handle_colon_end(p)
}

pub fn process_strings_to_paths(strings: Vec<String>) -> PathBuf {
    strings.into_iter().map(process_path).collect()
}

pub fn process_patterns(rule: &mut Rule, patterns: &[String]) -> RulesList {
    patterns
        .iter()
        .map(|pattern| extract_rule_with_pattern(rule, pattern))
        .collect()
}

pub fn map_patterns_to_rules(rule: &mut Rule) -> anyhow::Result<RulesList> {
    match rule.patterns {
        None => Ok(vec![rule.clone()]),
        Some(ref patterns) => Ok(process_patterns(&mut rule.clone(), patterns)),
    }
}

pub fn extract_rule_with_pattern(rule: &mut Rule, pattern: &str) -> Rule {
    rule.pattern = Some(pattern.to_string());
    let mut derived_rule = rule.clone();
    derived_rule.patterns = None;
    derived_rule
}

pub fn process_and_append_rule(rules: &mut RulesList, new_rules: &mut Vec<Rule>) -> anyhow::Result<()> {
    for rule in rules {
        let processed_rules = map_patterns_to_rules(rule)?;
        new_rules.extend(processed_rules);
    }
    Ok(())
}

pub fn process_rules(mappings: &mut RulesList, result_mappings: &mut Vec<Rule>) -> anyhow::Result<()> {
    process_and_append_rule(mappings, result_mappings)
}

pub fn process_and_append_rules(roots: Vec<RulesList>, new_rules: &mut Vec<Rule>) -> anyhow::Result<()> {
    let roots_with_indices = roots.into_iter().enumerate();
    for (idx, root) in roots_with_indices {
        for mut map in root {
            if map.root == 0 {
                map.root = idx;
            }
            process_and_append_rule(&mut vec![map], new_rules)?;
        }
    }
    Ok(())
}
