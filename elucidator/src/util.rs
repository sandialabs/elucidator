use crate::error::ElucidatorError;

type Result<T, E = ElucidatorError> = std::result::Result<T, E>;
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Buffer<'a> {
    /// The current position of the buffer cursor
    position: usize,
    /// The underlying data slice
    slice: &'a [u8],
}

impl<'a> Buffer<'a> {
    /// Make a new Buffer new a slice
    pub(crate) fn new(slice: &'a [u8]) -> Self {
        Buffer { position: 0, slice }
    }
    /// Make a new vector of n elements new current position
    pub(crate) fn grab(&mut self, n: usize) -> Result<Vec<u8>> {
        let curr_pos = self.position;
        if self.position + n > self.slice.len() {
            // Advance to end so that all future calls fail
            self.position = self.slice.len();
            Err(ElucidatorError::BufferSizing {
                expected: n,
                found: (self.slice.len() - curr_pos),
            })
        } else {
            self.position += n;
            Ok(self.slice[curr_pos..(curr_pos + n)].to_vec())
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn simple_ok() {
        let array = [1, 2, 3, 4];
        let mut buffer = Buffer::new(&array);
        let expected = Ok(array.to_vec());
        assert_eq!(expected, buffer.grab(4));
    }

    #[test]
    fn series_of_chunks_ok() {
        let array = [1, 2, 3, 4, 5, 6, 7, 8, 9];
        let mut buffer = Buffer::new(&array);
        let expected = Ok(array[..3].to_vec());
        assert_eq!(expected, buffer.grab(3));
        let expected = Ok(array[3..4].to_vec());
        assert_eq!(expected, buffer.grab(1));
        let expected = Ok(array[4..].to_vec());
        assert_eq!(expected, buffer.grab(5));
    }

    #[test]
    fn simple_err() {
        let array = [];
        let mut buffer = Buffer::new(&array);
        let expected = Err(ElucidatorError::BufferSizing {
            expected: 4,
            found: 0,
        });
        assert_eq!(expected, buffer.grab(4));
    }

    #[test]
    fn off_by_one_err() {
        let array = [1];
        let mut buffer = Buffer::new(&array);
        let expected = Err(ElucidatorError::BufferSizing {
            expected: 2,
            found: 1,
        });
        assert_eq!(expected, buffer.grab(2));
    }
}
