use bytes::BytesMut;

// Cursor for the BytesMut
#[derive(Debug)]
pub struct Cursor<'a> {
    inner: &'a mut BytesMut,
    pos: usize,
}

impl<'a> Cursor<'a> {
    pub fn new(inner: &'a mut BytesMut) -> Self {
        Cursor {
            inner,
            pos: 0,
        }
    }

    pub fn into_inner(&mut self) -> BytesMut {
        self.inner.split()
    }

    pub const fn position(&self) -> usize {
        self.pos
    }

    pub fn set_position(&mut self, pos: usize) {
        self.pos = pos;
    }

    pub fn reset(&mut self) {
        self.pos = 0;
    }

    // Len of the inner value
    #[allow(clippy::len_without_is_empty)]
    pub fn len(&self) -> usize {
        self.inner.len()
    }

    pub fn remaining(&self) -> &[u8] {
        self.inner[self.pos..].as_ref()
    }

    // split at current position and reset
    pub fn split_at_current_pos(&mut self) -> BytesMut {
        let pos = self.pos;
        self.reset();
        self.inner.split_to(pos)
    }
}

impl AsRef<[u8]> for Cursor<'_> {
    fn as_ref(&self) -> &[u8] {
        self.inner
    }
}

impl AsMut<BytesMut> for Cursor<'_> {
    fn as_mut(&mut self) -> &mut BytesMut {
        self.inner
    }
}
