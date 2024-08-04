
use bincode::{
    error::{DecodeError, EncodeError},
    Decode, Encode,
};
use tokio_tungstenite::tungstenite::Message;

#[derive(Debug, Clone, Encode, Decode, PartialEq)]
pub enum RPC {
    /// buffer operations  these are all read and write operaition
    RequestReadBuffer {
        path: String,
    },
    EditBuffer {
        changes: Vec<Vec<u8>>,
    },
    MoveCursor {
        path: String,
        position: usize,
        line: usize,
    },
    /// `.` should refer to the Current Working Directory
    SendTreeFileStructure {
        tree: Vec<String>,
    },
    /// send file to the client  
    SendFile {
        path: String,
        file: Vec<u8>, // this could be a automerge tree
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
    // string requests
    AddUsername(String),
    // Errors
    Error(String),
    // End Connection
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
