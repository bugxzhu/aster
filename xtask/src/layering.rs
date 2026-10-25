//! Automated layering check (06 section 1.2, 02 section 11.5). Uses the resolved normal
//! (non-dev, non-build) dependency graph from `cargo metadata`, so transitive dependencies
//! are checked, not only the direct ones.

use serde_json::Value;
use std::collections::{HashMap, HashSet, VecDeque};

/// A package name and the names of its normal dependencies.
pub type Graph = HashMap<String, Vec<String>>;

pub struct Rule {
    pub package: &'static str,
    pub reason: &'static str,
    pub forbidden: fn(&str) -> bool,
}

pub fn rules() -> Vec<Rule> {
    vec![
        Rule {
            package: "aster-ui-model",
            reason: "UI model must not depend on any rendering backend or application",
            forbidden: |n| {
                n.contains("slint")
                    || n.starts_with("aster-backend-")
                    || n.starts_with("file-manager-")
            },
        },
        Rule {
            package: "file-manager-core",
            reason: "business core must not depend on any UI backend or application shell",
            forbidden: |n| {
                n.contains("slint")
                    || n.starts_with("aster-backend-")
                    || n == "file-manager-aster"
                    || n == "file-manager-slint"
            },
        },
        Rule {
            package: "aster-backend-text",
            reason: "text backend must stay independent of Slint and of the application",
            forbidden: |n| {
                n.contains("slint") || n.starts_with("file-manager-") || n == "aster-backend-slint"
            },
        },
        Rule {
            package: "aster-backend-slint",
            reason: "Slint adapter must not depend on the application shell",
            forbidden: |n| n.starts_with("file-manager-"),
        },
    ]
}

/// Builds the graph from `cargo metadata --format-version 1` output.
pub fn graph_from_metadata(meta: &Value) -> Result<Graph, String> {
    let packages = meta["packages"]
        .as_array()
        .ok_or("metadata has no packages")?;
    let id_to_name: HashMap<String, String> = packages
        .iter()
        .filter_map(|p| {
            Some((
                p["id"].as_str()?.to_string(),
                p["name"].as_str()?.to_string(),
            ))
        })
        .collect();
    let nodes = meta["resolve"]["nodes"]
        .as_array()
        .ok_or("metadata has no resolve graph")?;
    let mut graph = Graph::new();
    for node in nodes {
        let id = node["id"].as_str().ok_or("node without id")?;
        let name = id_to_name.get(id).cloned().ok_or("unknown package id")?;
        let mut deps = Vec::new();
        for dep in node["deps"].as_array().unwrap_or(&Vec::new()) {
            let is_normal = dep["dep_kinds"]
                .as_array()
                .map(|kinds| kinds.iter().any(|k| k["kind"].is_null()))
                .unwrap_or(false);
            if !is_normal {
                continue;
            }
            if let Some(dep_id) = dep["pkg"].as_str() {
                if let Some(dep_name) = id_to_name.get(dep_id) {
                    deps.push(dep_name.clone());
                }
            }
        }
        graph.insert(name, deps);
    }
    Ok(graph)
}

/// Returns one message per violated rule. The message names the shortest dependency path from
/// the rule's package to the first forbidden package, which is the evidence to fix.
pub fn violations(graph: &Graph, rules: &[Rule]) -> Vec<String> {
    let mut out = Vec::new();
    for rule in rules {
        if !graph.contains_key(rule.package) {
            out.push(format!(
                "{}: package not found in dependency graph",
                rule.package
            ));
            continue;
        }
        if let Some(path) = shortest_forbidden_path(graph, rule) {
            out.push(format!(
                "{}: {} (path: {})",
                rule.package,
                rule.reason,
                path.join(" -> ")
            ));
        }
    }
    out
}

fn shortest_forbidden_path(graph: &Graph, rule: &Rule) -> Option<Vec<String>> {
    let mut parent: HashMap<&str, &str> = HashMap::new();
    let mut seen: HashSet<&str> = HashSet::new();
    let mut queue: VecDeque<&str> = VecDeque::new();
    seen.insert(rule.package);
    queue.push_back(rule.package);
    while let Some(current) = queue.pop_front() {
        for dep in graph.get(current).map(|v| v.as_slice()).unwrap_or(&[]) {
            if !seen.insert(dep.as_str()) {
                continue;
            }
            parent.insert(dep.as_str(), current);
            if (rule.forbidden)(dep) {
                let mut path = vec![dep.clone()];
                let mut node = dep.as_str();
                while let Some(p) = parent.get(node) {
                    path.push(p.to_string());
                    node = p;
                }
                path.reverse();
                return Some(path);
            }
            queue.push_back(dep.as_str());
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    fn g(edges: &[(&str, &[&str])]) -> Graph {
        edges
            .iter()
            .map(|(k, v)| (k.to_string(), v.iter().map(|s| s.to_string()).collect()))
            .collect()
    }

    fn rule(package: &'static str) -> Vec<Rule> {
        vec![Rule {
            package,
            reason: "test",
            forbidden: |n| n.contains("slint"),
        }]
    }

    #[test]
    fn transitive_forbidden_dependency_is_reported() {
        let graph = g(&[("app-model", &["mid"]), ("mid", &["slint"]), ("slint", &[])]);
        let v = violations(&graph, &rule("app-model"));
        assert_eq!(v.len(), 1, "{v:?}");
        assert!(v[0].ends_with("(path: app-model -> mid -> slint)"), "{v:?}");
    }

    #[test]
    fn clean_graph_has_no_violations() {
        let graph = g(&[("app-model", &["serde"]), ("serde", &[])]);
        assert!(violations(&graph, &rule("app-model")).is_empty());
    }

    #[test]
    fn one_violation_per_rule_with_shortest_path() {
        let graph = g(&[
            ("app-model", &["mid", "slint"]),
            ("mid", &["slint"]),
            ("slint", &["i-slint-core"]),
            ("i-slint-core", &[]),
        ]);
        let v = violations(&graph, &rule("app-model"));
        assert_eq!(v.len(), 1, "{v:?}");
        assert!(v[0].ends_with("(path: app-model -> slint)"), "{v:?}");
    }

    #[test]
    fn cycles_do_not_loop_forever() {
        let graph = g(&[("a", &["b"]), ("b", &["a", "slint"]), ("slint", &[])]);
        assert_eq!(violations(&graph, &rule("a")).len(), 1);
    }

    #[test]
    fn missing_package_is_a_violation() {
        assert_eq!(violations(&g(&[]), &rule("nope")).len(), 1);
    }

    #[test]
    fn metadata_parsing_ignores_dev_dependencies() {
        let meta: Value = serde_json::from_str(
            r#"{
              "packages": [
                {"id": "p1", "name": "app-model"},
                {"id": "p2", "name": "slint"}
              ],
              "resolve": {"nodes": [
                {"id": "p1", "deps": [
                  {"pkg": "p2", "dep_kinds": [{"kind": "dev", "target": null}]}
                ]},
                {"id": "p2", "deps": []}
              ]}
            }"#,
        )
        .unwrap();
        let graph = graph_from_metadata(&meta).unwrap();
        assert!(
            graph["app-model"].is_empty(),
            "dev dependency must be ignored"
        );
        assert!(violations(&graph, &rule("app-model")).is_empty());
    }

    #[test]
    fn metadata_parsing_keeps_normal_dependencies() {
        let meta: Value = serde_json::from_str(
            r#"{
              "packages": [{"id": "p1", "name": "app-model"}, {"id": "p2", "name": "slint"}],
              "resolve": {"nodes": [
                {"id": "p1", "deps": [{"pkg": "p2", "dep_kinds": [{"kind": null, "target": null}]}]},
                {"id": "p2", "deps": []}
              ]}
            }"#,
        )
        .unwrap();
        let graph = graph_from_metadata(&meta).unwrap();
        assert_eq!(graph["app-model"], vec!["slint".to_string()]);
    }
}
