use bincode::{
    error::{DecodeError, EncodeError},
    Decode, Encode,
};
use tokio_tungstenite::tungstenite::Message;
use serde::{Serialize, Deserialize};

use crate::server::connection::Priviledge;

#[derive(Debug, Clone, Encode, Decode, PartialEq)]
#[derive(Serialize, Deserialize)]
pub enum RPC {
    /// Response to a connection request
    ResConnect{
        username: String, // server username
        files: Vec<String>,
        emty_dirs: Vec<String>,
        priviledge: Priviledge,
    },
    /// inform the client that the priviledge that (his/her) privilege has been changed
    ChangePriviledge{
        priviledge: Priviledge,
    },
    /// buffer operations these are all read and write operaition
    ReqBufferTree {
        path: String,
    },
    /// `.` should refer to the Current Working Directory
    /// send file to the client  
    ResSendFile {
        path: String,
        file: Vec<u8>, // this could be a automerge tree
    },
    EditBuffer {
        path: String,
        changes: Vec<Vec<u8>>,
    },
    ReqMoveCursor {
        path: String,
        position: usize,
    },
    ResMoveCursor {
        username: String,
        path: String,
        position: usize,
    },
    RequestMark {
        path: String,
        s_position: usize,
        e_position: usize,
    },
    ResMark {
        path: String,
        s_position: usize,
        e_position: usize,
        username: String,
    },
    // Directory system operations
    CreateDirectory {
        path: String,
    },
    DeleteDirectory {
        path: String,
    },
    MoveDirectory {
        path: String,
        new_path: String,
    },
    // file system operations
    CreateFile {
        path: String,
    },
    DeleteFile {
        path: String,
    },
    MoveFile {
        path: String,
        new_path: String,
    },
    RequestSaveFile {
        path: String,
    },
    ResFileSaved{
        path: String,
    },

    AddUsername(String),
    Error(String),
    // this is a simple selection of a file
    // also it doesn't support multiple selection as not all editors support it
}

impl RPC {
    const CONFIG: bincode::config::Configuration = bincode::config::standard();

    /// encode the RPC to a slice of bytes
    pub fn encode(&self) -> Result<Message, EncodeError> {
        Ok(Message::binary(bincode::encode_to_vec(
            self.clone(),
            Self::CONFIG,
        )?))
    }
    /// decode the slice of bytes to RPC
    pub fn decode(encoded: &[u8]) -> Result<Self, DecodeError> {
        Ok(bincode::decode_from_slice(encoded, Self::CONFIG)?.0)
    }
}
