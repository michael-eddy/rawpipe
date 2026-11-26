use serde::Serialize;

use std::io::Write;
use std::fmt;
use std::fmt::Debug;

type Blake3Hasher = blake3::Hasher;
const HASH_SIZE: usize = 32;
pub type OpHash = [u8;HASH_SIZE];

#[derive(Clone)]
pub struct OpHasher {
  hash: Blake3Hasher,
}
impl OpHasher {
  pub fn new() -> OpHasher {
    OpHasher {
      hash: Blake3Hasher::new(),
    }
  }
  pub fn finish(&self) -> OpHash {
	*self.hash.finalize().as_bytes()
  }
}
impl Debug for OpHasher {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "OpHasher {{ {:?} }}", self.finish())
  }
}

impl Write for OpHasher {
  fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
    self.hash.update(buf);
    Ok(buf.len())
  }
  fn flush(&mut self) -> std::io::Result<()> {Ok(())}
}

impl OpHasher {
  pub fn hash_serialized<T>(&mut self, obj: &T) where T: Serialize {
    postcard::to_io(obj, self).unwrap();
  }
}
