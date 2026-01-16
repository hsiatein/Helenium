use anyhow::Context;
use anyhow::Result;
use heleny_proto::HealthStatus;
use heleny_proto::KernelHealth;
use std::collections::HashMap;
use std::collections::HashSet;
use tracing::debug;

pub struct DepsRelation {
    deps_map: HashMap<String, HashSet<String>>,
    rev_map: HashMap<String, HashSet<String>>,
    all_deps_map: HashMap<String, HashSet<String>>,
    all_rev_map: HashMap<String, HashSet<String>>,
    _init_seqs: HashMap<String, Vec<String>>,
    _stop_seqs: HashMap<String, Vec<String>>,
    init_cache: Option<HashMap<String, HashSet<String>>>,
    stop_cache: Option<HashMap<String, HashSet<String>>>,
}

impl DepsRelation {
    pub fn new(deps_map: HashMap<String, HashSet<String>>) -> Result<Self> {
        if !deps_exist(&deps_map) {
            return Err(anyhow::anyhow!("服务依赖里含有未知服务名"));
        }
        let rev_map = cal_rev_map(&deps_map);
        let mut all_deps_map = HashMap::new();
        let mut all_rev_map = HashMap::new();
        let mut init_seqs = HashMap::new();
        let mut stop_seqs = HashMap::new();
        let order = cal_order(deps_map.clone())?;
        for name in deps_map.keys() {
            // 依赖相关
            let all_deps = find_reachable_nodes(name.clone(), &deps_map);
            let init_seq = order
                .iter()
                .cloned()
                .filter(|name| all_deps.contains(name))
                .collect::<Vec<String>>();
            init_seqs.insert(name.clone(), init_seq);
            all_deps_map.insert(name.clone(), all_deps);
            // 反向依赖相关
            let all_rev = find_reachable_nodes(name.clone(), &rev_map);
            let stop_seq = order
                .iter()
                .rev()
                .cloned()
                .filter(|name| all_rev.contains(name))
                .collect::<Vec<String>>();
            stop_seqs.insert(name.clone(), stop_seq);
            all_rev_map.insert(name.clone(), all_rev);
        }
        Ok(Self {
            deps_map,
            rev_map,
            all_deps_map,
            all_rev_map,
            _init_seqs: init_seqs,
            _stop_seqs: stop_seqs,
            init_cache: None,
            stop_cache: None,
        })
    }

    /// 准备所有应用的缓存
    pub fn prepare_all_services(
        &mut self,
        health: KernelHealth,
        init: bool,
    ) -> Result<HashSet<String>> {
        let want_op: HashSet<String> = health.services.keys().cloned().collect();
        self.prepare_services(want_op, health, init)
    }

    /// 刷新缓存
    pub fn refresh_cache(&mut self, finish: &str, init: bool) -> Result<HashSet<String>> {
        let mut cache = match init {
            true => self.init_cache.take().context("未初始化 Init 缓存")?,
            false => self.stop_cache.take().context("未初始化 Stop 缓存")?,
        };
        debug_assert!(
            cache.remove(finish).is_none(),
            "按理cache不应包含finish, 前一步初始化时就移除了"
        );
        cache.iter_mut().for_each(|(_name, pre)| {
            pre.remove(finish);
        });
        let (can_op, need_op): (HashMap<_, _>, HashMap<_, _>) =
            cache.into_iter().partition(|(_name, pre)| pre.is_empty());
        match init {
            true => self.init_cache = Some(need_op),
            false => self.stop_cache = Some(need_op),
        };
        Ok(can_op.into_keys().collect())
    }

    /// 准备操作若干个应用的缓存
    pub fn prepare_services(
        &mut self,
        want_op: HashSet<String>,
        health: KernelHealth,
        init: bool,
    ) -> Result<HashSet<String>> {
        let (op, status) = match init {
            true => ("初始化", HealthStatus::Healthy),
            false => ("关闭", HealthStatus::Stopped),
        };
        debug!("想要{} {} 个应用", op, want_op.len());
        let want_op = self.prepare_cache(want_op, init)?;
        let (already_op, need_op): (HashMap<_, HashSet<_>>, HashMap<_, HashSet<_>>) =
            want_op.into_iter().partition(|(name, _pre)| {
                health
                    .services
                    .get(name)
                    .expect("不应出现 Health 没有的名字")
                    .0
                    == status
            });
        debug!("已经{}: {:?}, 需要{}: {:?}", op, already_op, op, need_op);
        let already_op: HashSet<String> = already_op.into_keys().collect();
        let need_op: HashMap<String, HashSet<String>> = need_op
            .into_iter()
            .map(|(name, pre)| (name, &pre - &already_op))
            .collect();
        let (can_op, need_op): (HashMap<_, HashSet<_>>, HashMap<_, HashSet<_>>) =
            need_op.into_iter().partition(|(_name, pre)| pre.is_empty());
        debug!("可以{}: {:?}, 暂时无法{}: {:?}", op, can_op, op, need_op);
        match init {
            true => self.init_cache = Some(need_op),
            false => self.stop_cache = Some(need_op),
        }
        Ok(can_op.into_keys().collect())
    }

    /// 输入所有想初始化/关闭的服务名字, 返回涉及到的所有服务名字对应的依赖表或被依赖表, init=true代表初始化, 否则代表关闭
    pub fn prepare_cache(
        &self,
        names: HashSet<String>,
        init: bool,
    ) -> Result<HashMap<String, HashSet<String>>> {
        let (all_map, normal_map) = match init {
            true => (&self.all_deps_map, &self.deps_map),
            false => (&self.all_rev_map, &self.rev_map),
        };
        let all_deps: HashSet<String> = names
            .into_iter()
            .try_fold(HashSet::new(), |mut current, name| {
                let all_deps = match all_map.get(&name) {
                    Some(all_deps) => all_deps.to_owned(),
                    None => return None,
                };
                current.extend(all_deps);
                current.insert(name);
                Some(current)
            })
            .context("没有对应服务名, 生成初始化缓存失败")?;
        let init_cache: HashMap<String, HashSet<String>> = normal_map
            .iter()
            .filter_map(|(name, dep)| match all_deps.contains(name) {
                true => Some((name.clone(), dep.clone())),
                false => None,
            })
            .collect();
        Ok(init_cache)
    }
}

/// 检查所有依赖是否存在
fn deps_exist(deps_map: &HashMap<String, HashSet<String>>) -> bool {
    !deps_map
        .values()
        .flatten()
        .any(|dep| !deps_map.contains_key(dep))
}

/// 计算反向依赖
fn cal_rev_map(deps_map: &HashMap<String, HashSet<String>>) -> HashMap<String, HashSet<String>> {
    let mut rev_map: HashMap<String, HashSet<String>> = deps_map
        .keys()
        .cloned()
        .into_iter()
        .map(|name| (name, HashSet::new()))
        .collect();
    for (name, deps) in deps_map {
        for dep in deps {
            rev_map
                .get_mut(dep)
                .expect("经过了依赖存在性检查, 不应该有未知依赖")
                .insert(name.clone());
        }
    }
    rev_map
}

/// 计算所有依赖
fn find_reachable_nodes(
    name: String,
    deps_map: &HashMap<String, HashSet<String>>,
) -> HashSet<String> {
    let mut all_deps = HashSet::new();
    let mut stack = vec![&name];
    while let Some(current) = stack.pop() {
        if let Some(deps) = deps_map.get(current) {
            for dep in deps {
                if all_deps.insert(dep.clone()) {
                    stack.push(dep);
                }
            }
        }
    }
    assert!(!all_deps.contains(&name));
    all_deps
}

/// 计算依赖顺序
fn cal_order(mut dag_map: HashMap<String, HashSet<String>>) -> Result<Vec<String>> {
    let mut order = Vec::new();
    let mut last_len = 0;
    while last_len != dag_map.len() {
        last_len = dag_map.len();
        let (new, remain): (
            HashMap<String, HashSet<String>>,
            HashMap<String, HashSet<String>>,
        ) = dag_map.into_iter().partition(|(_, deps)| deps.len() == 0);
        let new = new.into_keys().collect::<HashSet<String>>();
        dag_map = remain
            .into_iter()
            .map(|(k, deps)| (k, &deps - &new))
            .collect();
        order.extend(new);
    }
    if dag_map.len() == 0 {
        Ok(order)
    } else {
        Err(anyhow::anyhow!("有循环依赖或未知依赖 {:?}", dag_map.keys()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use std::collections::HashSet;

    // 辅助函数：快速创建 HashSet
    fn set(deps: Vec<String>) -> HashSet<String> {
        deps.into_iter().collect()
    }

    #[test]
    fn test_linear_dependency() {
        // 依赖链：db -> network -> kernel
        let mut dag = HashMap::new();
        dag.insert("kernel".to_string(), set(vec!["network".to_string()]));
        dag.insert("network".to_string(), set(vec!["db".to_string()]));
        dag.insert("db".to_string(), set(vec![]));

        let result = cal_order(dag).unwrap();
        println!("{:?}", result);
        // 检查绝对顺序
        assert_eq!(result.len(), 3);
        assert_eq!(result[0], "db");
        assert_eq!(result[1], "network");
        assert_eq!(result[2], "kernel");
    }

    #[test]
    fn test_parallel_dependency() {
        // 依赖树：A -> [B, C], B -> D, C -> D
        // D 必须第一，B和C顺序随机但必须在D之后，A必须最后
        let mut dag = HashMap::new();
        dag.insert("A".to_string(), set(vec!["B".to_string(), "C".to_string()]));
        dag.insert("B".to_string(), set(vec!["D".to_string()]));
        dag.insert("C".to_string(), set(vec!["D".to_string()]));
        dag.insert("D".to_string(), set(vec![]));

        let result = cal_order(dag).unwrap();
        println!("{:?}", result);
        // 验证相对位置
        let pos = |name| result.iter().position(|x| x == name).unwrap();

        assert!(pos("D") < pos("B"));
        assert!(pos("D") < pos("C"));
        assert!(pos("B") < pos("A"));
        assert!(pos("C") < pos("A"));
    }

    #[test]
    fn test_circular_dependency() {
        // 循环依赖：A -> B, B -> A
        let mut dag = HashMap::new();
        dag.insert("A".to_string(), set(vec!["B".to_string()]));
        dag.insert("B".to_string(), set(vec!["A".to_string()]));

        let result = cal_order(dag);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("有循环依赖"));
    }

    #[test]
    fn test_complex_mixed() {
        // 混合场景：某些服务无依赖，某些有深度依赖
        let mut dag = HashMap::new();
        dag.insert("S1".to_string(), set(vec![]));
        dag.insert("S2".to_string(), set(vec![]));
        dag.insert(
            "S3".to_string(),
            set(vec!["S1".to_string(), "S2".to_string()]),
        );
        dag.insert("S4".to_string(), set(vec!["S3".to_string()]));

        let result = cal_order(dag).unwrap();
        println!("{:?}", result);
        let pos = |name| result.iter().position(|x| x == name).unwrap();
        assert!(pos("S1") < pos("S3"));
        assert!(pos("S2") < pos("S3"));
        assert!(pos("S3") < pos("S4"));
    }

    #[test]
    fn test_fail_case_cross_generational_dependency() {
        let mut dag = HashMap::new();

        // 依赖关系：
        // L0: Base (无依赖)
        // L1: Mid  (依赖 Base)
        // L2: Top  (同时依赖 Mid 和 Base) -> 重点在这里！

        dag.insert("Base".to_string(), set(vec![]));
        dag.insert("Mid".to_string(), set(vec!["Base".to_string()]));
        dag.insert(
            "Top".to_string(),
            set(vec!["Mid".to_string(), "Base".to_string()]),
        );

        let result = cal_order(dag);
        println!("{:?}", result);
        // 你的当前实现在这里可能会返回 Err，或者 Top 的顺序不对
        // 理想结果应该是 ["Base", "Mid", "Top"]
        assert!(
            result.is_ok(),
            "应该能处理跨级依赖，但失败了: {:?}",
            result.err()
        );

        let order = result.unwrap();
        assert_eq!(order, vec!["Base", "Mid", "Top"]);
    }

    #[test]
    fn test_fail_case_missing_dependency() {
        let mut dag = HashMap::new();

        // S1 依赖了一个不存在的服务 "Ghost"
        // 按照逻辑，S1 永远不应该启动，最后应该报错输出 "S1"
        dag.insert("S1".to_string(), set(vec!["Ghost".to_string()]));

        let result = cal_order(dag);

        // 如果你的代码处理不当，可能会陷入死循环，或者返回空的 Ok
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("S1"), "错误信息应该包含未启动的服务 S1");
    }
}
