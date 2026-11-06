#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Key {
    Align,
    Shaping,
    Input,
    Tables,
    Mouse,
    UpdateCheck,
    Record,
}

pub(crate) struct Spec {
    pub(crate) key: Key,
    pub(crate) name: &'static str,
    pub(crate) label: &'static str,
    pub(crate) values: &'static [(&'static str, &'static str)],
}

const ON_OFF: &[(&str, &str)] = &[("true", "شغال"), ("false", "مقفول")];

pub(crate) const SPECS: [Spec; 7] = [
    Spec {
        key: Key::Align,
        name: "align",
        label: "المحاذاة",
        values: &[
            ("prose", "الكلام كله"),
            ("arabic", "العربي بس"),
            ("off", "مقفولة"),
        ],
    },
    Spec {
        key: Key::Shaping,
        name: "shaping",
        label: "تشكيل الحروف",
        values: &[("auto", "تلقائي"), ("full", "مرآة"), ("plain", "الترمنال")],
    },
    Spec {
        key: Key::Input,
        name: "input",
        label: "ترتيب البرنامج",
        values: &[("auto", "تلقائي"), ("logical", "منطقي"), ("visual", "مرئي")],
    },
    Spec {
        key: Key::Tables,
        name: "tables",
        label: "عكس الجداول",
        values: ON_OFF,
    },
    Spec {
        key: Key::Mouse,
        name: "mouse",
        label: "تحويل الماوس",
        values: ON_OFF,
    },
    Spec {
        key: Key::UpdateCheck,
        name: "update_check",
        label: "فحص التحديث",
        values: ON_OFF,
    },
    Spec {
        key: Key::Record,
        name: "record",
        label: "سجّل كل جلسة",
        values: ON_OFF,
    },
];

pub(crate) fn spec(key: Key) -> &'static Spec {
    SPECS.iter().find(|s| s.key == key).unwrap_or(&SPECS[0])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_key_has_a_spec_with_values() {
        for spec in &SPECS {
            assert_eq!(super::spec(spec.key).name, spec.name);
            assert!(spec.values.len() >= 2);
        }
    }
}
