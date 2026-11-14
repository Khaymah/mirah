pub(crate) type TabId = u32;

pub(crate) enum Event {
    Output(TabId, Vec<u8>),
    Input(Vec<u8>),
    OutputClosed(TabId),
    ChildExited(TabId, u32),
    Resize,
    #[cfg_attr(
        windows,
        allow(dead_code, reason = "only Unix signals stop mirah from outside")
    )]
    Stop(u32),
    UpdateFound(String),
    UpdateStep(crate::chrome::update::Step, Option<u8>),
    UpdateDone(Result<String, String>),
}
