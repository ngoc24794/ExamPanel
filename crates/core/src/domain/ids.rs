//! Strongly-typed identifier newtypes.

use serde::{Deserialize, Serialize};
use std::fmt;

macro_rules! define_id {
    ($name:ident, $doc:expr) => {
        #[doc = $doc]
        #[derive(
            Debug,
            Clone,
            Copy,
            PartialEq,
            Eq,
            PartialOrd,
            Ord,
            Hash,
            Default,
            Serialize,
            Deserialize,
        )]
        #[serde(transparent)]
        pub struct $name(pub i64);

        impl ts_rs::TS for $name {
            type WithoutGenerics = Self;
            type OptionInnerType = Self;
            fn name(_: &ts_rs::Config) -> String {
                "number".to_string()
            }
            fn decl(_: &ts_rs::Config) -> String {
                String::new()
            }
            fn inline(_: &ts_rs::Config) -> String {
                "number".to_string()
            }
            fn dependencies(_: &ts_rs::Config) -> Vec<ts_rs::Dependency> {
                vec![]
            }
        }

        impl $name {
            #[inline]
            #[must_use]
            pub const fn new(id: i64) -> Self {
                Self(id)
            }

            #[inline]
            #[must_use]
            pub const fn value(self) -> i64 {
                self.0
            }
        }

        impl From<i64> for $name {
            #[inline]
            fn from(v: i64) -> Self {
                Self(v)
            }
        }

        impl From<$name> for i64 {
            #[inline]
            fn from(id: $name) -> Self {
                id.0
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "{}", self.0)
            }
        }
    };
}

define_id!(CampusId, "Identifier for a Campus.");
define_id!(TeacherId, "Identifier for a Teacher.");
define_id!(GradeId, "Identifier for a Grade.");
define_id!(SchoolYearId, "Identifier for a SchoolYear.");
define_id!(ExamId, "Identifier for an Exam term.");
define_id!(SubjectId, "Identifier for a Subject.");
define_id!(PlanId, "Identifier for a generated Plan.");
define_id!(LockId, "Identifier for a manual Lock override.");

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_id_display_and_conversion() {
        let cid = CampusId::from(42);
        assert_eq!(cid.value(), 42);
        assert_eq!(i64::from(cid), 42);
        assert_eq!(format!("{cid}"), "42");
    }

    #[test]
    fn test_id_serde_roundtrip() {
        let tid = TeacherId(99);
        let serialized = serde_json::to_string(&tid).unwrap();
        assert_eq!(serialized, "99");
        let deserialized: TeacherId = serde_json::from_str(&serialized).unwrap();
        assert_eq!(deserialized, tid);
    }
}
