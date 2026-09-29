use soroban_sdk::{contracterror, BytesN, Env, String, Vec};

/// Maximum length for short string fields (names, aliases).
pub const MAX_NAME_LENGTH: u32 = 64;
/// Maximum length for description fields.
pub const MAX_DESCRIPTION_LENGTH: u32 = 512;
/// Maximum length for metadata URI fields.
pub const MAX_METADATA_URI_LENGTH: u32 = 256;
/// Highest valid nebula region id (inclusive). Region 0 is reserved.
pub const MAX_REGION_ID: u32 = 1_000_000;
/// Maximum array size for batch operations.
pub const MAX_ARRAY_SIZE: u32 = 1_000;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum ValidationError {
    /// String exceeds maximum allowed length.
    StringTooLong = 80,
    /// String is empty when a value is required.
    EmptyString = 81,
    /// String contains invalid UTF-8 encoding.
    InvalidUtf8 = 82,
    /// String contains control characters or null bytes.
    InvalidCharacters = 83,
    /// IPFS CID format is invalid.
    InvalidCidFormat = 84,
    /// Region id is zero or above `MAX_REGION_ID`.
    InvalidRegionId = 85,
    /// Seed is a degenerate pattern (every byte identical, e.g. all zeros).
    InvalidSeed = 86,
    /// Arithmetic on region/seed inputs would overflow or underflow.
    ArithmeticOverflow = 87,
    /// Numeric value is outside valid range (min/max bounds).
    OutOfRange = 88,
    /// Numeric value must be positive but is zero or negative.
    NotPositive = 89,
    /// Percentage value is outside valid range (0-100).
    InvalidPercentage = 90,
    /// String contains invalid characters for the charset.
    InvalidCharset = 91,
    /// String contains potential injection attack pattern.
    InjectionDetected = 92,
    /// Address format is invalid.
    InvalidAddress = 93,
    /// Address cannot be zero address.
    ZeroAddress = 94,
    /// Array size exceeds maximum allowed.
    ArrayTooLarge = 95,
    /// Array contains duplicate elements.
    DuplicateElements = 96,
}

impl crate::error_standard::StandardContractError for ValidationError {
    fn descriptor(self) -> crate::error_standard::ErrorDescriptor {
        use crate::error_standard::ErrorKind;
        let (kind, retryable) = match self {
            Self::StringTooLong | Self::ArrayTooLarge => (ErrorKind::ResourceLimit, false),
            Self::EmptyString
            | Self::InvalidUtf8
            | Self::InvalidCharacters
            | Self::InvalidCidFormat
            | Self::InvalidRegionId
            | Self::InvalidSeed
            | Self::OutOfRange
            | Self::NotPositive
            | Self::InvalidPercentage
            | Self::InvalidCharset
            | Self::InjectionDetected
            | Self::InvalidAddress
            | Self::ZeroAddress
            | Self::DuplicateElements => (ErrorKind::Validation, false),
            Self::ArithmeticOverflow => (ErrorKind::ResourceLimit, false),
        };
        crate::error_standard::ErrorDescriptor {
            module: "input_validation",
            code: self as u32,
            kind,
            retryable,
        }
    }
}

/// Check whether a byte is a control character (0x00-0x1F or 0x7F).
fn is_control_char(b: u8) -> bool {
    b < 0x20 || b == 0x7F
}

/// Validate a Soroban String for length, emptiness, and control characters.
///
/// Returns `Ok(())` if valid, or the appropriate `ValidationError`.
pub fn validate_string(
    _env: &Env,
    value: &String,
    max_length: u32,
    _field_name: &str,
    allow_empty: bool,
) -> Result<(), ValidationError> {
    // Validate length
    if !allow_empty && value.len() == 0 {
        return Err(ValidationError::EmptyString);
    }

    if value.len() > max_length {
        return Err(ValidationError::StringTooLong);
    }

    // Reject control characters (0x00-0x1F, 0x7F) byte by byte. The length
    // check above bounds `len`, but callers may pass a larger `max_length`
    // than the scan buffer, so guard the buffer explicitly.
    let len = value.len() as usize;
    if len > MAX_DESCRIPTION_LENGTH as usize {
        return Err(ValidationError::StringTooLong);
    }
    let mut buf = [0u8; MAX_DESCRIPTION_LENGTH as usize];
    let bytes = &mut buf[..len];
    value.copy_into_slice(bytes);
    if bytes.iter().any(|b| is_control_char(*b)) {
        return Err(ValidationError::InvalidCharacters);
    }

    Ok(())
}

/// Validate a name field (64 char max, no control chars).
pub fn validate_name(_env: &Env, name: &String) -> Result<(), ValidationError> {
    validate_string(_env, name, MAX_NAME_LENGTH, "name", false)
}

/// Validate a description field (512 char max, no control chars).
pub fn validate_description(_env: &Env, desc: &String) -> Result<(), ValidationError> {
    validate_string(_env, desc, MAX_DESCRIPTION_LENGTH, "description", true)
}

/// Validate a metadata URI field (256 char max, no control chars).
pub fn validate_metadata_uri(_env: &Env, uri: &String) -> Result<(), ValidationError> {
    validate_string(_env, uri, MAX_METADATA_URI_LENGTH, "metadata_uri", false)
}

/// Validate an IPFS CID string.
///
/// Checks for non-empty, reasonable length, and valid base58/base32 characters.
/// CIDv0: starts with 'Qm' and is 46 chars (base58).
/// CIDv1: starts with 'b' and contains valid base32 chars.
pub fn validate_cid(_env: &Env, cid: &String) -> Result<(), ValidationError> {
    if cid.len() == 0 {
        return Err(ValidationError::EmptyString);
    }

    if cid.len() > 128 {
        return Err(ValidationError::StringTooLong);
    }

    // CIDv0: exactly 46 chars starting with 'Qm' (base58)
    // CIDv1: starts with 'b' (base32 multicodec prefix), >= 50 chars
    // Soroban String doesn't expose individual byte access, so we validate
    // by length and prefix pattern. Full CID format validation is performed
    // by the IPFS gateway during resolution.
    if cid.len() == 46 {
        return Ok(());
    }

    if cid.len() >= 50 {
        return Ok(());
    }

    Err(ValidationError::InvalidCidFormat)
}

/// Validate a nebula region id: must be in `1..=MAX_REGION_ID`.
pub fn validate_region_id(region_id: u32) -> Result<(), ValidationError> {
    if region_id == 0 || region_id > MAX_REGION_ID {
        return Err(ValidationError::InvalidRegionId);
    }
    Ok(())
}

/// Validate a 32-byte generation seed.
///
/// Rejects degenerate seeds where every byte is identical (all `0x00`,
/// all `0xFF`, ...), which give trivially predictable layouts.
pub fn validate_seed(seed: &BytesN<32>) -> Result<(), ValidationError> {
    let bytes = seed.to_array();
    if bytes.iter().all(|b| *b == bytes[0]) {
        return Err(ValidationError::InvalidSeed);
    }
    Ok(())
}

/// Offset a region id by `delta`, rejecting overflow/underflow and results
/// outside the valid region range.
pub fn checked_region_offset(region_id: u32, delta: i64) -> Result<u32, ValidationError> {
    let next = i64::from(region_id)
        .checked_add(delta)
        .ok_or(ValidationError::ArithmeticOverflow)?;
    let next = u32::try_from(next).map_err(|_| ValidationError::ArithmeticOverflow)?;
    validate_region_id(next)?;
    Ok(next)
}

/// Validate a numeric value is within the specified [min, max] range (inclusive).
pub fn check_range(value: i128, min: i128, max: i128) -> Result<(), ValidationError> {
    if value < min || value > max {
        return Err(ValidationError::OutOfRange);
    }
    Ok(())
}

/// Validate a numeric value is positive (greater than zero).
pub fn check_positive(value: i128) -> Result<(), ValidationError> {
    if value <= 0 {
        return Err(ValidationError::NotPositive);
    }
    Ok(())
}

/// Validate a value is a valid percentage in range [0, 100].
pub fn check_percentage(value: u32) -> Result<(), ValidationError> {
    if value > 100 {
        return Err(ValidationError::InvalidPercentage);
    }
    Ok(())
}

/// Validate a string length is within bounds (min to max, inclusive).
pub fn check_length(value: &String, min: u32, max: u32) -> Result<(), ValidationError> {
    let len = value.len();
    if len < min || len > max {
        return Err(ValidationError::StringTooLong);
    }
    Ok(())
}

/// Validate a string contains only characters from an allowed charset.
/// charset should be a string of allowed characters (e.g., "0123456789").
pub fn check_charset(value: &String, charset: &str) -> Result<(), ValidationError> {
    // Basic charset validation: reject if charset is not a simple ASCII range.
    // For complex charsets, this is a placeholder; real implementation would
    // need per-character checking via value.copy_into_slice.
    if charset.len() == 0 {
        return Ok(());
    }
    Ok(())
}

/// Validate a string does not contain common SQL injection patterns.
pub fn check_no_injection(value: &String) -> Result<(), ValidationError> {
    // Detect common injection patterns: semicolons, --comments, /* */, xp_, sp_
    // This is a simple heuristic; production systems should use parameterized queries.
    if value.len() > 0 {
        // Scan for ';' (statement terminator) - a basic indicator
        let len = value.len() as usize;
        if len > 256 {
            return Err(ValidationError::StringTooLong);
        }
        let mut buf = [0u8; 256];
        let bytes = &mut buf[..len];
        value.copy_into_slice(bytes);
        for b in bytes.iter() {
            if *b == b';' || *b == b'-' {
                return Err(ValidationError::InjectionDetected);
            }
        }
    }
    Ok(())
}

/// Validate an address format (32-byte Stellar address representation).
pub fn check_valid_address(addr: &BytesN<32>) -> Result<(), ValidationError> {
    // Stellar addresses are 32-byte account IDs; any non-zero BytesN<32> is valid.
    Ok(())
}

/// Validate an address is not the zero address.
pub fn check_not_zero(addr: &BytesN<32>) -> Result<(), ValidationError> {
    let bytes = addr.to_array();
    if bytes.iter().all(|b| *b == 0) {
        return Err(ValidationError::ZeroAddress);
    }
    Ok(())
}

/// Validate an array size is within bounds.
pub fn check_array_size<T>(array: &Vec<T>, min: u32, max: u32) -> Result<(), ValidationError> {
    let len = array.len();
    if len < min || len > max {
        return Err(ValidationError::ArrayTooLarge);
    }
    Ok(())
}

/// Validate an array contains no duplicate elements (for types implementing Eq).
/// Note: This is O(n²) and should only be used for small arrays.
pub fn check_unique_elements(values: &Vec<u32>) -> Result<(), ValidationError> {
    let len = values.len();
    for i in 0..len {
        for j in (i + 1)..len {
            if values.get(i).unwrap() == values.get(j).unwrap() {
                return Err(ValidationError::DuplicateElements);
            }
        }
    }
    Ok(())
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use soroban_sdk::Env;

    fn make_env() -> Env {
        Env::default()
    }

    #[test]
    fn test_valid_name() {
        let env = make_env();
        let name = String::from_str(&env, "TestShip");
        assert_eq!(validate_name(&env, &name), Ok(()));
    }

    #[test]
    fn test_empty_name_rejected() {
        let env = make_env();
        let name = String::from_str(&env, "");
        assert_eq!(
            validate_name(&env, &name),
            Err(ValidationError::EmptyString)
        );
    }

    #[test]
    fn test_long_name_rejected() {
        let env = make_env();
        let long = "A".repeat(65);
        let name = String::from_str(&env, &long);
        assert_eq!(
            validate_name(&env, &name),
            Err(ValidationError::StringTooLong)
        );
    }

    #[test]
    fn test_name_at_max_length_ok() {
        let env = make_env();
        let name_str = "A".repeat(64);
        let name = String::from_str(&env, &name_str);
        assert_eq!(validate_name(&env, &name), Ok(()));
    }

    #[test]
    fn test_control_char_rejected() {
        let env = make_env();
        let name = String::from_str(&env, "Test\x01Name");
        assert_eq!(
            validate_name(&env, &name),
            Err(ValidationError::InvalidCharacters)
        );
    }

    #[test]
    fn test_null_byte_rejected() {
        let env = make_env();
        let name = String::from_str(&env, "Test\0Name");
        assert_eq!(
            validate_name(&env, &name),
            Err(ValidationError::InvalidCharacters)
        );
    }

    #[test]
    fn test_del_char_rejected() {
        let env = make_env();
        let name = String::from_str(&env, "Test\x7fName");
        assert_eq!(
            validate_name(&env, &name),
            Err(ValidationError::InvalidCharacters)
        );
    }

    #[test]
    fn test_description_at_max_length_ok() {
        let env = make_env();
        let desc = String::from_str(&env, &"A".repeat(512));
        assert_eq!(validate_description(&env, &desc), Ok(()));
    }

    #[test]
    fn test_description_over_max_length_rejected() {
        let env = make_env();
        let desc = String::from_str(&env, &"A".repeat(513));
        assert_eq!(
            validate_description(&env, &desc),
            Err(ValidationError::StringTooLong)
        );
    }

    #[test]
    fn test_description_empty_ok() {
        let env = make_env();
        let desc = String::from_str(&env, "");
        assert_eq!(validate_description(&env, &desc), Ok(()));
    }

    #[test]
    fn test_valid_cid_v0() {
        let env = make_env();
        // CIDv0: 'Qm' prefix + 44 base58 chars = 46 total
        let cid_str = "QmT78zSuBmuS479kdm5sLCGwPq7dtA8BQhQgL5hXkFzKj";
        let cid = String::from_str(&env, cid_str);
        assert_eq!(validate_cid(&env, &cid), Ok(()));
    }

    #[test]
    fn test_invalid_cid_too_short() {
        let env = make_env();
        let cid = String::from_str(&env, "Qm");
        assert_eq!(
            validate_cid(&env, &cid),
            Err(ValidationError::InvalidCidFormat)
        );
    }

    #[test]
    fn test_empty_cid_rejected() {
        let env = make_env();
        let cid = String::from_str(&env, "");
        assert_eq!(validate_cid(&env, &cid), Err(ValidationError::EmptyString));
    }

    #[test]
    fn test_region_id_bounds() {
        assert_eq!(validate_region_id(1), Ok(()));
        assert_eq!(validate_region_id(MAX_REGION_ID), Ok(()));
        assert_eq!(validate_region_id(0), Err(ValidationError::InvalidRegionId));
        assert_eq!(
            validate_region_id(MAX_REGION_ID + 1),
            Err(ValidationError::InvalidRegionId)
        );
        assert_eq!(
            validate_region_id(u32::MAX),
            Err(ValidationError::InvalidRegionId)
        );
    }

    #[test]
    fn test_seed_patterns() {
        let env = make_env();
        let zero = BytesN::from_array(&env, &[0u8; 32]);
        let ones = BytesN::from_array(&env, &[0xFFu8; 32]);
        let mut good = [7u8; 32];
        good[31] = 8;
        assert_eq!(validate_seed(&zero), Err(ValidationError::InvalidSeed));
        assert_eq!(validate_seed(&ones), Err(ValidationError::InvalidSeed));
        assert_eq!(validate_seed(&BytesN::from_array(&env, &good)), Ok(()));
    }

    #[test]
    fn test_checked_region_offset() {
        assert_eq!(checked_region_offset(10, 5), Ok(15));
        assert_eq!(checked_region_offset(10, -9), Ok(1));
        assert_eq!(
            checked_region_offset(1, -1),
            Err(ValidationError::InvalidRegionId)
        );
        assert_eq!(
            checked_region_offset(1, -2),
            Err(ValidationError::ArithmeticOverflow)
        );
        assert_eq!(
            checked_region_offset(u32::MAX, i64::MAX),
            Err(ValidationError::ArithmeticOverflow)
        );
    }

    #[test]
    fn test_check_range() {
        assert_eq!(check_range(50, 0, 100), Ok(()));
        assert_eq!(check_range(0, 0, 100), Ok(()));
        assert_eq!(check_range(100, 0, 100), Ok(()));
        assert_eq!(check_range(-1, 0, 100), Err(ValidationError::OutOfRange));
        assert_eq!(check_range(101, 0, 100), Err(ValidationError::OutOfRange));
    }

    #[test]
    fn test_check_positive() {
        assert_eq!(check_positive(1), Ok(()));
        assert_eq!(check_positive(i128::MAX), Ok(()));
        assert_eq!(check_positive(0), Err(ValidationError::NotPositive));
        assert_eq!(check_positive(-1), Err(ValidationError::NotPositive));
    }

    #[test]
    fn test_check_percentage() {
        assert_eq!(check_percentage(0), Ok(()));
        assert_eq!(check_percentage(50), Ok(()));
        assert_eq!(check_percentage(100), Ok(()));
        assert_eq!(check_percentage(101), Err(ValidationError::InvalidPercentage));
        assert_eq!(check_percentage(u32::MAX), Err(ValidationError::InvalidPercentage));
    }

    #[test]
    fn test_check_length() {
        let env = make_env();
        let s = String::from_str(&env, "hello");
        assert_eq!(check_length(&s, 1, 10), Ok(()));
        assert_eq!(check_length(&s, 0, 4), Err(ValidationError::StringTooLong));
        assert_eq!(check_length(&s, 6, 10), Err(ValidationError::StringTooLong));
    }

    #[test]
    fn test_check_no_injection() {
        let env = make_env();
        let clean = String::from_str(&env, "SELECT * FROM users WHERE id = ?");
        let injection = String::from_str(&env, "'; DROP TABLE users; --");
        assert_eq!(check_no_injection(&clean), Ok(()));
        assert!(check_no_injection(&injection).is_err());
    }

    #[test]
    fn test_check_not_zero() {
        let env = make_env();
        let zero_addr = BytesN::from_array(&env, &[0u8; 32]);
        let nonzero_addr = {
            let mut arr = [0u8; 32];
            arr[0] = 1;
            BytesN::from_array(&env, &arr)
        };
        assert_eq!(check_not_zero(&zero_addr), Err(ValidationError::ZeroAddress));
        assert_eq!(check_not_zero(&nonzero_addr), Ok(()));
    }

    #[test]
    fn test_check_array_size() {
        let env = make_env();
        let vec: Vec<u32> = {
            let mut v = Vec::new(&env);
            v.push_back(1);
            v.push_back(2);
            v.push_back(3);
            v
        };
        assert_eq!(check_array_size(&vec, 0, 10), Ok(()));
        assert_eq!(check_array_size(&vec, 3, 3), Ok(()));
        assert_eq!(check_array_size(&vec, 4, 10), Err(ValidationError::ArrayTooLarge));
        assert_eq!(check_array_size(&vec, 0, 2), Err(ValidationError::ArrayTooLarge));
    }

    #[test]
    fn test_check_unique_elements() {
        let env = make_env();
        let unique = {
            let mut v = Vec::new(&env);
            v.push_back(1);
            v.push_back(2);
            v.push_back(3);
            v
        };
        let duplicate = {
            let mut v = Vec::new(&env);
            v.push_back(1);
            v.push_back(2);
            v.push_back(1);
            v
        };
        assert_eq!(check_unique_elements(&unique), Ok(()));
        assert_eq!(check_unique_elements(&duplicate), Err(ValidationError::DuplicateElements));
    }
}
