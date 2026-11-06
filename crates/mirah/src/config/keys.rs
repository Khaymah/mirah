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
    pub(crate) advanced: bool,
}

const ON_OFF: &[(&str, &str)] = &[("true", "تشغيل"), ("false", "إيقاف")];

pub(crate) const SPECS: [Spec; 7] = [
    Spec {
        key: Key::Align,
        name: "align",
        label: "المحاذاة",
        values: &[
            ("prose", "كل النص"),
            ("arabic", "العربي فقط"),
            ("off", "إيقاف"),
        ],
        advanced: false,
    },
    Spec {
        key: Key::Tables,
        name: "tables",
        label: "عكس الجداول",
        values: ON_OFF,
        advanced: false,
    },
    Spec {
        key: Key::Record,
        name: "record",
        label: "تسجيل كل جلسة",
        values: ON_OFF,
        advanced: false,
    },
    Spec {
        key: Key::UpdateCheck,
        name: "update_check",
        label: "البحث عن تحديثات",
        values: ON_OFF,
        advanced: false,
    },
    Spec {
        key: Key::Shaping,
        name: "shaping",
        label: "وصل الحروف",
        values: &[("auto", "تلقائي"), ("full", "مرآة"), ("plain", "الطرفية")],
        advanced: true,
    },
    Spec {
        key: Key::Input,
        name: "input",
        label: "ترتيب نص البرنامج",
        values: &[("auto", "تلقائي"), ("logical", "منطقي"), ("visual", "مرئي")],
        advanced: true,
    },
    Spec {
        key: Key::Mouse,
        name: "mouse",
        label: "تحويل نقرات الماوس",
        values: ON_OFF,
        advanced: true,
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
