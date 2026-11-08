//! The shared half of a checked string type: `as_str`, `Debug`,
//! `Display`, `AsRef<str>`, `FromStr` and serde through the type's own
//! `new`, which keeps its own error enum. `redacted` hides the text from
//! `Debug`. `no_display` leaves `Display` to the type.

macro_rules! validated_string {
    ($type:ty, $error:ty) => {
        impl std::fmt::Debug for $type {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                std::fmt::Debug::fmt(&self.0, f)
            }
        }

        validated_string!(@display $type);
        validated_string!(@common $type, $error);
    };
    ($type:ty, $error:ty, redacted) => {
        impl std::fmt::Debug for $type {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.debug_struct(stringify!($type))
                    .field("bytes", &self.0.len())
                    .finish()
            }
        }

        validated_string!(@display $type);
        validated_string!(@common $type, $error);
    };
    ($type:ty, $error:ty, no_display) => {
        impl std::fmt::Debug for $type {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                std::fmt::Debug::fmt(&self.0, f)
            }
        }

        validated_string!(@common $type, $error);
    };
    (@display $type:ty) => {
        impl std::fmt::Display for $type {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.write_str(&self.0)
            }
        }
    };
    (@common $type:ty, $error:ty) => {
        impl $type {
            pub fn as_str(&self) -> &str {
                &self.0
            }
        }

        impl AsRef<str> for $type {
            fn as_ref(&self) -> &str {
                &self.0
            }
        }

        impl std::str::FromStr for $type {
            type Err = $error;

            fn from_str(s: &str) -> Result<Self, Self::Err> {
                Self::new(s)
            }
        }

        impl serde::Serialize for $type {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: serde::Serializer,
            {
                serializer.serialize_str(&self.0)
            }
        }

        impl<'de> serde::Deserialize<'de> for $type {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                let value = <String as serde::Deserialize>::deserialize(deserializer)?;
                Self::new(value).map_err(serde::de::Error::custom)
            }
        }
    };
}
