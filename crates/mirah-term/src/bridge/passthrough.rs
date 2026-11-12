use crate::sequences::{CSI, OSC, ST};

const OSC_WINDOW_TITLE: &[u8] = b"2;";
const OSC_CLIPBOARD: &[u8] = b"52;";
const OSC_QUERY: &[u8] = b"?";

fn is_private_prefix(b: u8) -> bool {
    matches!(b, b'?' | b'>' | b'<' | b'=')
}

const WINDOW_REPORTS: std::ops::RangeInclusive<u16> = 11..=21;

pub(super) fn forwards_csi(i1: Option<u8>, i2: Option<u8>, params: &[&[u16]], c: char) -> bool {
    match (i1, i2, c) {
        (None | Some(b'>' | b'='), None, 'c')
        | (Some(b'>'), None, 'q')
        | (_, Some(b'$'), 'p')
        | (Some(b'$'), None, 'p') => true,
        (None, None, 't') => params
            .first()
            .and_then(|p| p.first())
            .is_some_and(|n| WINDOW_REPORTS.contains(n)),
        _ => false,
    }
}

pub(super) fn rebuild_csi(i1: Option<u8>, i2: Option<u8>, params: &[&[u16]], c: char) -> Vec<u8> {
    let mut out = CSI.as_bytes().to_vec();
    let mut suffix = Vec::new();
    for i in [i1, i2].into_iter().flatten() {
        if is_private_prefix(i) && out.len() == CSI.len() {
            out.push(i);
        } else {
            suffix.push(i);
        }
    }
    let only_default = params.len() == 1 && params[0] == [0];
    if !only_default {
        let joined: Vec<String> = params
            .iter()
            .map(|p| p.iter().map(u16::to_string).collect::<Vec<_>>().join(":"))
            .collect();
        out.extend_from_slice(joined.join(";").as_bytes());
    }
    out.extend_from_slice(&suffix);
    let mut buf = [0u8; 4];
    out.extend_from_slice(c.encode_utf8(&mut buf).as_bytes());
    out
}

fn osc(body: &[&[u8]]) -> Vec<u8> {
    let mut out = OSC.as_bytes().to_vec();
    body.iter().for_each(|part| out.extend_from_slice(part));
    out.extend_from_slice(ST.as_bytes());
    out
}

pub(super) fn osc_query(params: &[&[u8]]) -> Option<Vec<u8>> {
    params
        .contains(&OSC_QUERY)
        .then(|| osc(&[&params.join(&b';')]))
}

pub(super) fn window_title(title: &[u8]) -> Vec<u8> {
    osc(&[OSC_WINDOW_TITLE, title])
}

pub(super) fn clipboard(selection: &[u8], data: &[u8]) -> Vec<u8> {
    osc(&[OSC_CLIPBOARD, selection, b";", data])
}
