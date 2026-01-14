use anyhow::Result;
use heleny_proto::McpOutput;
use heleny_proto::McpToolManual;
use serde_json::json;
use std::process::Stdio;
use tokio::io::AsyncBufReadExt;
use tokio::io::AsyncWriteExt;
use tokio::io::BufReader;
use tokio::process;

#[tokio::test]
async fn test_process() -> Result<()> {
    let mut child = process::Command::new("docker")
        .arg("run")
        .arg("-i")
        .arg("--rm")
        .arg("mcp/duckduckgo")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;
    let mut child_stdin = child.stdin.take().unwrap();
    let child_stdout = child.stdout.take().unwrap();
    let child_stderr = child.stderr.take().unwrap();

    let init = json!({
    "jsonrpc":"2.0",
    "id":1,
    "method":"initialize",
    "params":{
        "protocolVersion":"2025-06-18",
        "capabilities":{},
        "clientInfo":{"name":"MyAgent","version":"0.1.0"}
    }
    });

    child_stdin.write_all(init.to_string().as_bytes()).await?;
    child_stdin.write_all(b"\n").await?;
    child_stdin.flush().await?;

    let mut out_lines = BufReader::new(child_stdout).lines();
    let mut err_lines = BufReader::new(child_stderr).lines();
    tokio::spawn(async move {
        while let Ok(Some(line)) = err_lines.next_line().await {
            eprintln!("[tool stderr] {}", line);
        }
    });

    while let Some(line) = out_lines.next_line().await? {
        let t = line.trim();
        if t.is_empty() {
            continue;
        }

        // 日志可能混进来：先尝试 parse
        if let Ok(v) = serde_json::from_str::<serde_json::Value>(t) {
            println!("JSON: {}", v);
            break;
        } else {
            eprintln!("LOG?: {}", t);
        }
    }
    child.kill().await?;
    Ok(())
}

#[tokio::test]
async fn test_process2() -> Result<()> {
    let mut child = process::Command::new("docker")
        .arg("run")
        .arg("-i")
        .arg("--rm")
        .arg("mcp/duckduckgo")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()?;
    let mut child_stdin = child.stdin.take().unwrap();
    let child_stdout = child.stdout.take().unwrap();

    let init = json!({
    "jsonrpc":"2.0",
    "id":0,
    "method":"initialize",
    "params":{
        "protocolVersion":"2025-06-18",
        "capabilities":{},
        "clientInfo":{"name":"MyAgent","version":"0.1.0"}
    }
    });

    child_stdin.write_all(init.to_string().as_bytes()).await?;
    child_stdin.write_all(b"\n").await?;
    child_stdin.flush().await?;

    let mut out_lines = BufReader::new(child_stdout).lines();

    let Some(line) = out_lines.next_line().await? else {
        return Err(anyhow::anyhow!("读取失败"));
    };
    let t = line.trim();
    let Ok(v) = serde_json::from_str::<serde_json::Value>(t) else {
        return Err(anyhow::anyhow!("转化json失败"));
    };
    let output: McpOutput = serde_json::from_value(v)?;
    println!("JSON: {:?}", output.result);

    let initialized = json!({"jsonrpc":"2.0","method":"notifications/initialized"});

    child_stdin
        .write_all(initialized.to_string().as_bytes())
        .await?;
    child_stdin.write_all(b"\n").await?;
    child_stdin.flush().await?;

    let tools_list = json!({"jsonrpc":"2.0","id":1,"method":"tools/list","params":{}});
    child_stdin
        .write_all(tools_list.to_string().as_bytes())
        .await?;
    child_stdin.write_all(b"\n").await?;
    child_stdin.flush().await?;

    let Some(line) = out_lines.next_line().await? else {
        return Err(anyhow::anyhow!("读取失败"));
    };
    let t = line.trim();
    let Ok(v) = serde_json::from_str::<serde_json::Value>(t) else {
        return Err(anyhow::anyhow!("转化json失败"));
    };
    let output: McpOutput = serde_json::from_value(v)?;
    let tools: Vec<McpToolManual> =
        serde_json::from_value(output.result.get("tools").unwrap().clone())?;
    println!("JSON: {:?}", tools);
    Ok(())
}
