use std::path::PathBuf;
use tracing_subscriber::EnvFilter;
use tracing_subscriber::fmt;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

/// 初始化 tracing 订阅者
/// 返回一个 guard，必须在 main 函数中一直持有它，否则日志无法写入文件
pub fn init_tracing(log_dir: PathBuf) -> tracing_appender::non_blocking::WorkerGuard {
    // 1. 设置过滤规则：默认显示 info 级别及以上的日志
    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("debug"));

    // 2. 配置控制台打印 (stdout)
    let formatting_layer = fmt::layer()
        .with_ansi(true) // 开启彩色输出
        .with_thread_ids(true) // 打印线程 ID，方便排查死锁
        .with_line_number(true);

    // 3. 配置滚动文件记录 (每日生成新文件)
    let file_appender = tracing_appender::rolling::daily(log_dir, "helenium.log");
    let (non_blocking_appender, guard) = tracing_appender::non_blocking(file_appender);
    let file_layer = fmt::layer()
        .with_writer(non_blocking_appender)
        .with_ansi(false) // 文件日志不需要彩色字符
        .with_line_number(true);

    // 4. 将所有层组合并注册到全局
    tracing_subscriber::registry()
        .with(env_filter)
        .with(formatting_layer)
        .with(file_layer)
        .init();

    // 返回 guard，防止后台线程被 drop
    guard
}
