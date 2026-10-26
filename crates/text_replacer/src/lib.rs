use std::collections::BTreeMap;
use config::config_section;
use serde::{Deserialize, Serialize};
use std::collections::{HashSet, VecDeque};


#[config_section("replacer")]
#[derive(Default, Debug)]
pub struct Replacer {
    pub rules: BTreeMap<String, Rule>, // in dependency graph form
    #[serde(skip)]
    rule_cache: BTreeMap<String, Rule>, // stored lazily, Replacer::replace stores expanded rules here in first usage
}

#[derive(Serialize, Deserialize, Clone, Default, Debug)]
pub struct Rule {
    pub map: BTreeMap<String, String>,
    pub dependencies: Vec<String>,
    #[serde(skip)]
    pub name: String,
}

impl Rule {
    pub fn replace(&self, text: &str) -> String {
        let mut result = String::with_capacity(text.len());
        let mut start = 0;

        for (index, ch) in text.char_indices() {
            if !ch.is_alphanumeric() && ch != '\'' {
                if start < index {
                    self.replace_word(&text[start..index], &mut result);
                }
                result.push(ch);
                start = index + ch.len_utf8();
            }
        }
        if start < text.len() {
            self.replace_word(&text[start..], &mut result);
        }
        result
    }

    fn replace_word(&self, word: &str, result: &mut String) {
        let key = word.to_lowercase(); // full Unicode lowercasing
        match self.map.get(&key) {
            Some(to) => result.push_str(to),
            None => result.push_str(word),
        }
    }
}

impl Replacer {
    pub fn replace(&mut self, rule: &str, text: &str) -> String {
        if !self.rule_cache.contains_key(rule) {
            let expanded = self.construct(rule.to_string());

            eprintln!("Constructed map contains im: {}", expanded.map.contains_key("im"));
            eprintln!("Constructed map size: {}", expanded.map.len());

            self.rule_cache.insert(rule.to_string(), expanded);
        }

        self.rule_cache[rule].replace(text)
    }

    pub fn all_rules(&self) -> Vec<&String> {
        self.rules.keys().collect()
    }

    // if that rule doens't exist, just return a dummy/default rule
    pub fn construct(&mut self, name: String) -> Rule {
        let mut rule = match self.rules.get(&name) {
            Some(r) => r.clone(),
            None => Rule::default(),
        };

        rule.name = name.clone();

        let mut visited = HashSet::new();
        let mut queue = VecDeque::new();

        visited.insert(name.clone());

        for dependency in &rule.dependencies {
            queue.push_back(dependency.clone());
        }

        while let Some(dependency) = queue.pop_front() {
            if !visited.insert(dependency.clone()) {
                continue;
            }

            let Some(dep) = self.rules.get(&dependency) else {
                continue;
            };

            rule.map.extend(dep.map.clone());

            for child in &dep.dependencies {
                queue.push_back(child.clone());
            }
        }

        rule.dependencies.clear();
        rule
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_replaceme(
        name: &str,
        dependencies: &[&str],
        rules: &[(&str, &str)],
    ) -> Rule {
        Rule {
            name: name.to_string(),
            dependencies: dependencies.iter().map(|s| s.to_string()).collect(),
            map: rules
                .iter()
                .map(|(from, to)| (from.to_string(), to.to_string()))
                .collect(),
        }
    }

    fn make_replacer(entries: Vec<Rule>) -> Replacer {
        Replacer {
            rules: entries
                .into_iter()
                .map(|r| (r.name.clone(), r))
                .collect(),
            rule_cache: BTreeMap::default(),
        }
    }

    #[test]
    fn merges_direct_dependencies() {
        let mut replacer = make_replacer(vec![
            make_replaceme("base", &[], &[("foo", "bar")]),
            make_replaceme("child", &["base"], &[("baz", "qux")]),
        ]);

        let result = replacer.construct("child".into());

        assert_eq!(result.map.len(), 2);
        assert_eq!(result.map["foo"], "bar");
        assert_eq!(result.map["baz"], "qux");
    }

    #[test]
    fn merges_transitive_dependencies() {
        let mut replacer = make_replacer(vec![
            make_replaceme("root", &[], &[("a", "b")]),
            make_replaceme("middle", &["root"], &[("c", "d")]),
            make_replaceme("leaf", &["middle"], &[("e", "f")]),
        ]);

        let result = replacer.construct("leaf".into());

        assert_eq!(result.map.len(), 3);
        assert_eq!(result.map["a"], "b");
        assert_eq!(result.map["c"], "d");
        assert_eq!(result.map["e"], "f");
    }

    #[test]
    fn handles_missing_dependencies() {
        let mut replacer = make_replacer(vec![
            make_replaceme("child", &["missing"], &[("foo", "bar")]),
        ]);

        let result = replacer.construct("child".into());

        assert_eq!(result.map.len(), 1);
        assert_eq!(result.map["foo"], "bar");
    }

    #[test]
    fn handles_circular_dependencies() {
        let mut replacer = make_replacer(vec![
            make_replaceme("a", &["b"], &[("foo", "bar")]),
            make_replaceme("b", &["a"], &[("baz", "qux")]),
        ]);

        let result = replacer.construct("a".into());

        assert_eq!(result.map.len(), 2);
        assert_eq!(result.map["foo"], "bar");
        assert_eq!(result.map["baz"], "qux");
    }

    #[test]
    fn dependency_overrides_duplicate_key() {
        let mut replacer = make_replacer(vec![
            make_replaceme("base", &[], &[("foo", "base_value")]),
            make_replaceme("child", &["base"], &[("foo", "child_value")]),
        ]);

        let result = replacer.construct("child".into());

        // Dependencies are merged after the child, so the dependency wins.
        assert_eq!(result.map["foo"], "base_value");
    }
}