use std::collections::HashMap;
use std::process::Stdio;

use anyhow::Context;
use anyhow::Result;
use serde::Deserialize;
use serde::Serialize;
use tokio::io::AsyncBufReadExt;
use tokio::io::AsyncWriteExt;
use tokio::io::BufReader;
use tokio::io::Lines;
use tokio::process::Child;
use tokio::process::ChildStdin;
use tokio::process::ChildStdout;
use tokio::process::{self};
use tracing::warn;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct HelenyProcessCommand {
    command: String,
    args: Vec<String>,
    #[serde(default)]
    env: HashMap<String, String>,
}

impl HelenyProcessCommand {
    pub async fn spawn(&self) -> Result<HelenyProcess> {
        let mut _cmd;
        let mut cmd;
        if ["npx"].contains(&self.command.as_str()) && cfg!(target_os = "windows") {
            _cmd = process::Command::new("cmd");
            cmd = &mut _cmd;
            cmd = cmd.arg("/c");
            cmd = cmd.arg(&self.command);
        }
        else {
            _cmd = process::Command::new(&self.command);
            cmd = &mut _cmd;
        }
        for arg in &self.args {
            cmd = cmd.arg(arg);
        }
        for (k, v) in &self.env {
            cmd = cmd.env(k, v);
        }
        match cmd.stdin(Stdio::piped()).stdout(Stdio::piped()).spawn() {
            Ok(child) => Ok(HelenyProcess::new(child)?),
            Err(e) => Err(anyhow::anyhow!("启动子进程 {:?} 失败: {}", self, e)),
        }
    }
}

#[derive(Debug)]
pub struct HelenyProcess {
    child: Child,
    stdin: ChildStdin,
    stdout_lines: Lines<BufReader<ChildStdout>>,
}

impl HelenyProcess {
    pub fn new(mut child: Child) -> Result<Self> {
        let stdin = child.stdin.take().context("获取 stdin 失败")?;
        let stdout = child.stdout.take().context("获取 stdout 失败")?;
        let stdout_lines: Lines<BufReader<ChildStdout>> = BufReader::new(stdout).lines();
        let process = Self {
            child,
            stdin,
            stdout_lines,
        };
        Ok(process)
    }

    pub async fn read(&mut self) -> Result<String> {
        let next_line = self.stdout_lines.next_line().await?;
        next_line.context("next line 为 None")
    }

    pub async fn write(&mut self, context: &str) -> Result<()> {
        self.stdin.write_all(context.as_bytes()).await?;
        self.stdin.write_all(b"\n").await?;
        self.stdin.flush().await?;
        Ok(())
    }
}

impl Drop for HelenyProcess {
    fn drop(&mut self) {
        if let Err(e) = self.child.start_kill() {
            warn!("关闭子进程失败: {}", e)
        };
    }
}
