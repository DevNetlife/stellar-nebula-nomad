/// Validation macros for common patterns.

/// Validate a numeric value is within a range and return early on error.
/// Expands to: if !(value >= min && value <= max) { return Err(OutOfRange) }
#[macro_export]
macro_rules! validate_range {
    ($value:expr, $min:expr, $max:expr) => {
        $crate::input_validation::check_range($value as i128, $min as i128, $max as i128)?
    };
}

/// Validate a percentage value is in [0, 100] and return early on error.
#[macro_export]
macro_rules! validate_percentage {
    ($value:expr) => {
        $crate::input_validation::check_percentage($value)?
    };
}

/// Validate a value is positive (> 0) and return early on error.
#[macro_export]
macro_rules! validate_positive {
    ($value:expr) => {
        $crate::input_validation::check_positive($value as i128)?
    };
}

/// Validate a string is not empty and return early on error.
#[macro_export]
macro_rules! validate_not_empty {
    ($value:expr) => {
        if $value.len() == 0 {
            return Err($crate::input_validation::ValidationError::EmptyString);
        }
    };
}

/// Validate a string has specified length bounds and return early on error.
#[macro_export]
macro_rules! validate_string_length {
    ($value:expr, $min:expr, $max:expr) => {
        $crate::input_validation::check_length(&$value, $min, $max)?
    };
}

/// Validate an address is not zero and return early on error.
#[macro_export]
macro_rules! validate_address_not_zero {
    ($addr:expr) => {
        $crate::input_validation::check_not_zero(&$addr)?
    };
}

/// Validate an array size is within bounds and return early on error.
#[macro_export]
macro_rules! validate_array_size {
    ($array:expr, $min:expr, $max:expr) => {
        $crate::input_validation::check_array_size(&$array, $min, $max)?
    };
}

/// Validate an array has no duplicate elements and return early on error.
#[macro_export]
macro_rules! validate_no_duplicates {
    ($array:expr) => {
        $crate::input_validation::check_unique_elements(&$array)?
    };
}

/// Validate a string does not contain injection patterns and return early on error.
#[macro_export]
macro_rules! validate_no_injection {
    ($value:expr) => {
        $crate::input_validation::check_no_injection(&$value)?
    };
}

/// Chainable validator pattern: collect multiple validations that must all pass.
/// Use by calling validator functions and collecting results.
/// Example: validate_chain!(check_positive(val1), check_percentage(val2))
#[macro_export]
macro_rules! validate_chain {
    ($($check:expr),+ $(,)?) => {{
        $(
            $check?;
        )+
        Ok::<(), $crate::input_validation::ValidationError>(())
    }};
}
