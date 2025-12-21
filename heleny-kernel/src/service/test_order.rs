use super::*;
use std::collections::{HashMap, HashSet};

// 辅助函数：快速创建 HashSet
fn set(deps: Vec<&'static str>) -> HashSet<&'static str> {
    deps.into_iter().collect()
}

#[test]
fn test_linear_dependency() {
    // 依赖链：db -> network -> kernel
    let mut dag = HashMap::new();
    dag.insert("kernel", set(vec!["network"]));
    dag.insert("network", set(vec!["db"]));
    dag.insert("db", set(vec![]));

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
    dag.insert("A", set(vec!["B", "C"]));
    dag.insert("B", set(vec!["D"]));
    dag.insert("C", set(vec!["D"]));
    dag.insert("D", set(vec![]));

    let result = cal_order(dag).unwrap();
    println!("{:?}", result);
    // 验证相对位置
    let pos = |name| result.iter().position(|&x| x == name).unwrap();

    assert!(pos("D") < pos("B"));
    assert!(pos("D") < pos("C"));
    assert!(pos("B") < pos("A"));
    assert!(pos("C") < pos("A"));
}

#[test]
fn test_circular_dependency() {
    // 循环依赖：A -> B, B -> A
    let mut dag = HashMap::new();
    dag.insert("A", set(vec!["B"]));
    dag.insert("B", set(vec!["A"]));

    let result = cal_order(dag);
    // println!("{:?}",result);
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("有循环依赖"));
}

#[test]
fn test_complex_mixed() {
    // 混合场景：某些服务无依赖，某些有深度依赖
    let mut dag = HashMap::new();
    dag.insert("S1", set(vec![]));
    dag.insert("S2", set(vec![]));
    dag.insert("S3", set(vec!["S1", "S2"]));
    dag.insert("S4", set(vec!["S3"]));

    let result = cal_order(dag).unwrap();
    println!("{:?}", result);
    let pos = |name| result.iter().position(|&x| x == name).unwrap();
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

    dag.insert("Base", set(vec![]));
    dag.insert("Mid", set(vec!["Base"]));
    dag.insert("Top", set(vec!["Mid", "Base"]));

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
    dag.insert("S1", set(vec!["Ghost"]));

    let result = cal_order(dag);

    // 如果你的代码处理不当，可能会陷入死循环，或者返回空的 Ok
    assert!(result.is_err());
    let err_msg = result.unwrap_err().to_string();
    assert!(err_msg.contains("S1"), "错误信息应该包含未启动的服务 S1");
}
