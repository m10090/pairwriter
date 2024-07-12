use bincode::{Encode, Decode};
#[derive(Debug,Clone,Encode,Decode, PartialEq)]
pub enum RPC {
    // buffer operations
    ReadBuffer {
        path: String,
    },
    WriteOnBuffer {
        path: String,
        line: u64,
        input: String,
        position: u64,
    },
    DeleteOnBuffer {
        path: String,
        position: u64,
        line: u64,
    },
    MoveCursor {
        path: String,
        position: u64,
        line: u64,
    },

    SendTreeFileStructure {
        vector: Vec<String>,
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
}
#[cfg(test)]
mod tests {
    use super::*;
    macro_rules! test_rpc {
        ($name:ident, $rpc_variant:expr) => {
            #[test]
            fn $name() {
                let rpc = $rpc_variant;
                let config = bincode::config::standard();
                let encoded: Vec<u8> = bincode::encode_to_vec(&rpc, config).unwrap();
                println!("{:?}", encoded);
                let (decoded, len): (RPC, usize) = bincode::decode_from_slice(&encoded[..], config).unwrap();
                println!("decoded: {:?} \n with size {:?}", decoded, len);
                assert_eq!(rpc, decoded);
            }
        };
    }
    // test for all rpc variants
    test_rpc!(test_rpc_send_tree, RPC::SendTreeFileStructure {
        vector: vec!["to/path/a".to_string(), "to/path/b".to_string()],
    });

    test_rpc!(test_rpc_read_file, RPC::ReadBuffer {
        path: "/to/path".to_string(),
    });

    test_rpc!(test_rpc_write_on_buffer, RPC::WriteOnBuffer {
        path: "/to/path".to_string(),
        position: 1,
        line: 1,
        input: "input".to_string(),
    });

    test_rpc!(test_rpc_delete_on_buffer, RPC::DeleteOnBuffer {
        path: "/to/path".to_string(),
        line: 1,
        position: 1,
    });

    test_rpc!(test_rpc_move_cursor, RPC::MoveCursor {
        path: "/to/path".to_string(),
        line: 1,
        position: 1,
    });

    test_rpc!(test_rpc_create_directory, RPC::CreateDirectory {
        path: "/to/path".to_string(),
    });

    test_rpc!(test_rpc_delete_directory, RPC::DeleteDirectory {
        path: "/to/path".to_string(),
    });

    test_rpc!(test_rpc_move_directory, RPC::MoveDirectory {
        path: "/to/path".to_string(),
        new_path: "/to/new/path".to_string(),
    });

    test_rpc!(test_rpc_create_file, RPC::CreateFile {
        path: "/to/path".to_string(),
    });

    test_rpc!(test_rpc_delete_file, RPC::DeleteFile {
        path: "/to/path".to_string(),
    });

    test_rpc!(test_rpc_move_file, RPC::MoveFile {
        path: "/to/path".to_string(),
        new_path: "/to/new/path".to_string(),
    });
    
    // testing bincode security
    #[test]
    fn test_rpc_security() {
        let rpc = RPC::ReadBuffer {
            path: "/to/path".to_string(),
        };
        let config = bincode::config::standard();
        let mut encoded: Vec<u8> = bincode::encode_to_vec(&rpc, config).unwrap();
        encoded[0] = 6; // changing the type of the rpc
        encoded[1] = 9; // increasing size of the string
        println!("{:?}", encoded);
        let (decode,len): (RPC,usize) = bincode::decode_from_slice(&encoded[..], config).unwrap_or_else(|e| {
            println!("{:?}", e);
            (RPC::ReadBuffer {
                path: "/to/path".to_string(),
            }, 2)
        });
        println!("decoded: {:?} \n with size {:?}", decode, len);
        assert_eq!(rpc, decode);
        assert_eq!(len, 2);
    }
}

