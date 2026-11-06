pub(crate) type TabId = u32;

pub(crate) enum Event {
    Output(TabId, Vec<u8>),
    Input(Vec<u8>),
    OutputClosed(TabId),
    ChildExited(TabId, u32),
    Resize,
    UpdateFound(String),
    UpdateDone(Result<String, String>),
}
