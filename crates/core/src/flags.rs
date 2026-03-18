use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct Flags {
  map: HashMap<String, bool>,
}

impl Flags {
  pub fn from_env(prefix: &str) -> Self {
    // Reads flags like `${prefix}FLAG_NAME=true`.
    let mut map = HashMap::new();
    for (k, v) in std::env::vars() {
      if let Some(rest) = k.strip_prefix(prefix) {
        let enabled = matches!(v.as_str(), "1" | "true" | "TRUE" | "yes" | "YES" | "on" | "ON");
        map.insert(rest.to_string(), enabled);
      }
    }
    Self { map }
  }

  pub fn enabled(&self, flag: &str) -> bool {
    self.map.get(flag).copied().unwrap_or(false)
  }
}

