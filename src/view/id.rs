#[derive(Copy, Clone, PartialEq, Eq)]
pub(crate) struct ViewId(u64);

pub(crate) struct ViewIdMaker(u64);
impl ViewIdMaker {
    pub(crate) fn new() -> ViewIdMaker {
        ViewIdMaker(0)
    }
    pub(crate) fn next_id(&mut self) -> ViewId {
        let id = ViewId(self.0);
        self.0 += 1;
        id
    }
}
