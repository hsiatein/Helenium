use std::path::PathBuf;

use tokio::sync::oneshot;

#[derive(Debug)]
pub enum FsServiceMessage {
    Read {
        path: PathBuf,
        feedback: oneshot::Sender<String>,
    },
    Write {
        path: PathBuf,
        content: String,
        feedback: oneshot::Sender<()>,
    },
    Update,
    List {
        dir: PathBuf,
        feedback: oneshot::Sender<Vec<PathBuf>>,
    },
    Load {
        path: PathBuf,
        feedback: oneshot::Sender<()>,
    },
    GetImage {
        path: PathBuf,
        feedback: oneshot::Sender<Vec<u8>>,
    },
}
