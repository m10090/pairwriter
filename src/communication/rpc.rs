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
        path: String,
        changes: Vec<Vec<u8>>,
    },
    ClientMoveCursor {
        path: String,
        position: usize,
    },
    ServerMoveCursor {
        username: String,
        path: String,
        position: usize,
    },
    /// `.` should refer to the Current Working Directory
    ServerSendTreeFileStructure {
        tree: Vec<String>,
    },
    /// send file to the client  
    ServerSendFile {
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
    ClientRequestSaveFile {
        path: String,
    },
    ServerFileSaved{
        path: String,
    },
    AddUsername(String),
    
    Error(String),
    // this is a simple selection of a file
    // also it doesn't support multiple selection as not all editors support it
    RequestMark {
        path: String,
        s_position: usize,
        e_position: usize,
    },
    Mark {
        path: String,
        s_position: usize,
        e_position: usize,
        username: String,
    },
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
