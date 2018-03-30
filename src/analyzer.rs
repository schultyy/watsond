use regex::Regex;

pub fn analyze(file: &str) -> Vec<String> {
  let mut analyzer_findings = vec!();

  for line in file.split("\n") {
    if is_match(line) {
      analyzer_findings.push(line.to_string());
    }
  }

  analyzer_findings
}

fn is_match(text: &str) -> bool {
    lazy_static! {
        static ref RE: Regex = Regex::new(r"ERROR").unwrap();
    }
    RE.is_match(text)
}
