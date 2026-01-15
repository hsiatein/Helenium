use std::path::PathBuf;

use heleny_proto::HelenyFile;
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
    TempFile {
        file: HelenyFile,
        file_ext: String,
        feedback: oneshot::Sender<PathBuf>,
    }
}
