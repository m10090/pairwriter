use std::io::{self, Error};

use automerge::{
    transaction::Transactable as _, Automerge, ChangeHash, ObjType, ReadDoc as _, Value, ROOT,
};
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

    fn bytes_to_head_history(bytes: Vec<Vec<[u8; 32]>>) -> Vec<Vec<ChangeHash>> {
        bytes
            .into_iter()
            .map(|x| x.into_iter().map(|y| ChangeHash(y)).collect())
            .collect()
    }
    pub(crate) fn new(
        automerge: Vec<u8>,
        heads_history: Vec<Vec<[u8; 32]>>,
        head_idx: usize,
    ) -> Self {
        let heads_history = Self::bytes_to_head_history(heads_history);
        Self {
            automerge: Automerge::load(automerge.as_slice()).unwrap(),
            head_idx,
            heads_history,
        }
    }
    /// this will update the crdt with the change
    /// this also takes into account the undo and redo
    pub(crate) fn update(
        &mut self,
        changes: &[u8],
        old_head_idx: usize,
        new_heads: &[Vec<[u8; 32]>],
    ) -> Result<(), Box<dyn std::error::Error>> {
        if !old_head_idx < self.heads_history.len() {
            return Err(Box::new(Error::new(
                std::io::ErrorKind::InvalidInput,
                "old_head_idx is out of bounds",
            )));
        }

        let new_heads = Self::bytes_to_head_history(new_heads.to_vec());
        let new_heads = new_heads.as_slice(); 
        // get the last common head after that there is changes
        let fork_head = self.heads_history[old_head_idx].as_slice();

        self.automerge = self.automerge.fork_at(fork_head)?;

        (&mut self.automerge).load_incremental(changes)?;

        self.heads_history = [&self.heads_history[..=old_head_idx], new_heads]
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
            Self::CONTENT,
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

    pub(crate) fn edit(
        &mut self,
        pos: Option<usize>,
        del: Option<isize>,
        text: &str,
    ) -> (Vec<u8>, usize, Vec<Vec<[u8; 32]>>) {
        let obj_id = self.automerge.get(ROOT, Self::CONTENT).unwrap().unwrap().1; // to do
        if self.head_idx < self.heads_history.len() - 1 {
            self.automerge = self
                .automerge
                .fork_at(self.heads_history[self.head_idx].as_slice())
                .unwrap();
        }
        {
            let mut tx = self.automerge.transaction();
            if pos.is_none() && del.is_none() {
                let _ = tx.update_text(&obj_id, text);
            } else {
                let _ = tx.splice_text(obj_id, pos.unwrap(), del.unwrap(), text);
            }
            tx.commit();
        }
        let changes = self
            .automerge
            .save_after(self.heads_history[self.head_idx].as_slice());

        [
            &self.heads_history[..=self.head_idx],
            &[self.automerge.get_heads()],
        ]
        .concat()
        .to_vec();
        self.head_idx = self.heads_history.len() - 1;
        let new_heads = self.heads_history[self.head_idx].clone();
        let new_heads = new_heads.into_iter().map(|x| x.0.clone()).collect();
        (changes, self.head_idx - 1, vec![new_heads])
    }

    fn get_heads_history(&self) -> Vec<Vec<[u8; 32]>> {
        self.heads_history
            .clone()
            .into_iter()
            .map(|x| x.into_iter().map(|y| y.0.clone()).collect())
            .collect()
    }
    pub(crate) fn save(&self) -> (Vec<u8>, Vec<Vec<[u8; 32]>>, usize) {
        (
            self.automerge.save(),
            self.get_heads_history(),
            self.head_idx,
        )
    }
}
