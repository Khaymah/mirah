pub(crate) enum Event {
    Output(Vec<u8>),
    Resize,
    ChildExited,
}
