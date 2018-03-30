use regex::RegexSet;
use std::collections::HashSet;

pub fn analyze(file: &str, analyzers: &HashSet<String>) -> Vec<String> {
  let mut analyzer_findings = vec!();

  let compiled_regexes = RegexSet::new(analyzers).unwrap();

  for line in file.split("\n") {
    let matches: Vec<_> = compiled_regexes.matches(line).into_iter().collect();
    if matches.len() > 0 {
      analyzer_findings.push(line.to_string());
    }
  }

  analyzer_findings
}
