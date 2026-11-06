pub(crate) enum Event {
    Output(Vec<u8>),
    Input(Vec<u8>),
    OutputClosed,
    ChildExited(u32),
    Resize,
    UpdateFound(String),
    UpdateDone(Result<String, String>),
}
