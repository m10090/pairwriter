use std::io::{self, Error};

use automerge::{Automerge, ChangeHash, ObjType, ReadDoc as _, Value, ROOT};

#[derive(Debug, Clone)]
pub(crate) struct Crdt {
    automerge: Automerge,
    heads_history: Vec<Vec<ChangeHash>>,
    head_idx: usize,
}

impl Crdt {
    const CONTENT: &'static str = "content";
    /// this should be used when reading the file for the first time
    pub(crate) fn open(automerge: Automerge) -> Self {
        let heads_history = vec![automerge.get_heads()];
        Self {
            automerge,
            heads_history,
            head_idx: 0,
        }
    }

    pub(crate) fn new(automerge: Automerge, heads: Vec<Vec<ChangeHash>>) -> Self {
        Self {
            automerge,
            head_idx: heads.len() - 1,
            heads_history: heads,
        }
    }
    /// this will update the crdt with the change
    /// this also takes into account the undo and redo
    pub(crate) fn update(
        &mut self,
        change: Vec<u8>,
        old_head_idx: usize,
        heads: Vec<Vec<ChangeHash>>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        if !old_head_idx < self.heads_history.len() {
            return Err(Box::new(Error::new(
                std::io::ErrorKind::InvalidInput,
                "old_head_idx is out of bounds",
            )));
        }

        // get the last common head after that there is changes
        let fork_head = self.heads_history[old_head_idx].as_slice();

        self.automerge = self.automerge.fork_at(fork_head)?;

        self.automerge.load_incremental(change.as_slice())?;

        self.heads_history = [&self.heads_history[..=old_head_idx], heads.as_slice()]
            .concat()
            .to_vec();
        self.head_idx = self.heads_history.len() - 1;
        Ok(())
    }

    pub(crate) fn undo(&mut self) {
        if self.head_idx == 0 {
            // do nothing
            return;
        }
        self.head_idx -= 1;
    }

    pub(crate) fn redo(&mut self) {
        if self.head_idx == self.heads_history.len() - 1 {
            // do nothing
            return;
        }
        self.head_idx += 1;
    }

    pub(crate) fn read(&self) -> Result<Vec<u8>, io::Error> {
        if self.heads_history.is_empty() || self.head_idx >= self.heads_history.len() {
            return Err(Error::new(
                std::io::ErrorKind::InvalidInput,
                "head_idx is out of bounds",
            ));
        }
        let buf = &self.automerge;
        match buf.get_at(
            ROOT,
            "content",
            self.heads_history[self.head_idx].as_slice(),
        ) {
            Ok(content) => {
                let (val, id) = content.unwrap();
                if val.is_object() && matches!(val, Value::Object(ObjType::Text)) {
                    Ok(buf
                        .text_at(id, self.heads_history[self.head_idx].as_slice())
                        .unwrap()
                        .as_bytes()
                        .to_vec())
                } else if val.is_bytes() {
                    Ok(val.to_bytes().unwrap().to_vec()) // there is some issue there
                                                         // Ok(vec![])
                } else {
                    Err(Error::new(
                        io::ErrorKind::InvalidData,
                        "The file could be corrupted",
                    ))
                }
            }
            Err(_) => Err(Error::new(
                io::ErrorKind::InvalidData,
                "The file is corrupted",
            )),
        }
    }
}
