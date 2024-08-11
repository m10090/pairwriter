use client_crdt::ClientFunc;

use crate::communication::rpc::RPC;

use super::*;
pub trait Transactions {
    fn handel_message(&mut self, rpc: RPC) -> Result<(), ()>;
}
impl<T> Transactions for T
where
    T: ClientFunc,
{
    fn handel_message(&mut self, rpc: RPC) -> Result<(), ()> {
        match rpc {
            RPC::RequestReadBuffer { path  }=>{
                todo!()
            }
            RPC::RequestSaveFile { path} => {
                todo!()
            }
            RPC::MoveFile { path, new_path } => {
                todo!("self.move_file(&path, &new_path)")
            }
            RPC::DeleteFile { path } => {
                todo!( "self.rm_file(&path)" )
            }
            RPC::Error(error) => {
                eprintln!("Error: {:?}", error)
            }
            RPC::SendFile { path, file} => {
                todo!("self.save_file(&path, &buf)")
            }
            RPC::EditBuffer { path, changes } => {
                todo!()
            }
            RPC::MoveCursor { path, position } =>{ 
                todo!()
            }
            RPC::DeleteDirectory { path  } => {
                self.rm_dir(&path)
            }

            
            
        }
        Ok(())
    }
}
