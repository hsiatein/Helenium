use std::collections::{HashMap, HashSet};

pub struct DepsRelation {
    deps_map: HashMap<&'static str, HashSet<&'static str>>,
    rev_map: HashMap<&'static str, HashSet<&'static str>>,
    init_order: HashMap<&'static str, Vec<&'static str>>,
    stop_order: HashMap<&'static str, Vec<&'static str>>,
}

pub fn deps_exist(deps_map: &HashMap<&'static str, HashSet<&'static str>>) -> bool {
    !deps_map
        .values()
        .flatten()
        .any(|dep| !deps_map.contains_key(dep))
}

pub fn cal_rev_map(
    deps_map: &HashMap<&'static str, HashSet<&'static str>>,
) -> HashMap<&'static str, HashSet<&'static str>> {
    let mut rev_map: HashMap<&str, HashSet<&str>> = deps_map
        .keys()
        .into_iter()
        .map(|&name| (name, HashSet::new()))
        .collect();
    for (name, deps) in deps_map {
        for dep in deps {
            rev_map
                .get_mut(dep)
                .expect("经过了依赖存在性检查, 不应该有未知依赖")
                .insert(&name);
        }
    }
    rev_map
}
