use std::ops::{Index, IndexMut};


#[derive(Debug, Default)]
pub struct Storage {
    data: Vec<u8>,
}

impl Index<usize> for Storage {
    type Output = u8;

    fn index(&self, index: usize) -> &Self::Output {
        if index + 1 > self.data.len() {
            &0_u8
        } else {
            &self.data[index]
        }
    }
}

impl IndexMut<usize> for Storage {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        if index + 1 > self.data.len() {
            self.data.resize(index + 1, 0);
        }
        &mut self.data[index]
    }
}
