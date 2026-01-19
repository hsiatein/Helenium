use std::{path::PathBuf, time::SystemTime};
use tokio::sync::oneshot;
use uuid::Uuid;

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
    GetOriginImage {
        path: PathBuf,
        feedback: oneshot::Sender<Vec<u8>>,
    },
    GetImage {
        path: PathBuf,
        feedback: oneshot::Sender<Vec<u8>>,
    },
    TempFile {
        dir_name: String,
        file_name: String,
        data: Vec<u8>,
        feedback: oneshot::Sender<PathBuf>,
    },
    WriteBytes {
        path: PathBuf,
        data: Vec<u8>,
    },
    ReadBytes {
        path: PathBuf,
        feedback: oneshot::Sender<Vec<u8>>,
    },
    NewThumbnail {
        id:Uuid,
        origin_path: PathBuf,
        last_modified: SystemTime,
        thumbnail:Vec<u8>,
    }
}
