use regex::RegexSet;
use std::collections::HashSet;

#[derive(Debug, Deserialize, Serialize)]
pub struct Finding {
    pub line_number: usize,
    pub line: String
}

pub fn analyze(file: &str, analyzers: &HashSet<String>) -> Vec<Finding> {
  let mut analyzer_findings = vec!();

  let compiled_regexes = RegexSet::new(analyzers).unwrap();

  for (index, line) in file.split("\n").enumerate() {
    let matches: Vec<_> = compiled_regexes.matches(line).into_iter().collect();
    if matches.len() > 0 {
      analyzer_findings.push(Finding{
        line_number: index + 1,
        line: line.to_string()
      });
    }
  }

  analyzer_findings
}
