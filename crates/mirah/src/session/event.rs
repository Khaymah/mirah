pub(crate) enum Event {
    Output(Vec<u8>),
    OutputClosed,
    ChildExited(u32),
    Resize,
}
