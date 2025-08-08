// For config values

#[macro_export]
macro_rules! value_of {
    ($config:expr, $field:ident) => {
        $config
            .$field
            .clone()
            .ok_or_else(|| anyhow!(format!(
            "Mandatory field undefined or invalid: '{}'",
            stringify!($field)
        )))
    };
}

#[macro_export]
macro_rules! value_of_if {
    ($condition:expr, $config:expr, $field:ident) => {
        if $condition {
            value_of!($config, $field)
        }
        else {
            Ok(Default::default())
        }
    };
}

#[macro_export]
macro_rules! generate_with_defaults {
    {$struct_name:ident, $($field_name:ident,)* } => {
        impl $struct_name {
            pub fn with_defaults(&self, defaults: Self) -> Self {
                Self {
                    $($field_name: self.$field_name.clone().or(defaults.$field_name),)*
                }
            }
        }
    };
}
