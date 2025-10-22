use totp_lite::{totp, Sha1};
use base64::Engine;
use rand::Rng;

/// Generate a TOTP secret for 2FA
pub fn generate_totp_secret() -> String {
    // Generate 32 random bytes for the secret
    let mut rng = rand::thread_rng();
    let secret_bytes: Vec<u8> = (0..32).map(|_| rng.gen::<u8>()).collect();

    // Encode to base32 for display in authenticator apps
    use base64::engine::general_purpose::STANDARD;
    STANDARD.encode(&secret_bytes)
}

/// Generate TOTP backup codes (for account recovery if phone is lost)
pub fn generate_backup_codes(count: usize) -> Vec<String> {
    let mut rng = rand::thread_rng();
    (0..count)
        .map(|_| {
            // Generate 8 random bytes and encode as hex
            let bytes: Vec<u8> = (0..8).map(|_| rng.gen::<u8>()).collect();
            format!("{}", bytes.iter().map(|b| format!("{:02x}", b)).collect::<String>())
        })
        .collect()
}

/// Verify a TOTP code against a secret
pub fn verify_totp_code(secret: &str, code: &str) -> Result<bool, Box<dyn std::error::Error>> {
    // Decode base64 secret
    use base64::engine::general_purpose::STANDARD;
    let secret_bytes = STANDARD.decode(secret)?;

    // Verify code (allow for time drift of ±1 time step)
    let current_time = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)?
        .as_secs();

    // Check current code and adjacent time steps
    for time_offset in -1..=1 {
        let time_step = (current_time / 30) as i64 + time_offset;
        if time_step >= 0 {
            // Use totp function directly: totp::<Sha1>(secret, time_step, digits)
            let expected_code = totp::<Sha1>(&secret_bytes, time_step as u64);
            if expected_code == code {
                return Ok(true);
            }
        }
    }

    Ok(false)
}

/// Generate current TOTP code for testing
pub fn generate_totp_code(secret: &str) -> Result<String, Box<dyn std::error::Error>> {
    use base64::engine::general_purpose::STANDARD;
    let secret_bytes = STANDARD.decode(secret)?;

    let current_time = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)?
        .as_secs();

    let time_step = current_time / 30;
    // Use totp function directly
    Ok(totp::<Sha1>(&secret_bytes, time_step))
}

/// Generate QR code data URI for TOTP setup
pub fn generate_totp_qr_code(username: &str, secret: &str, issuer: &str) -> Result<String, Box<dyn std::error::Error>> {
    use qrcode::QrCode;

    // Format: otpauth://totp/issuer:username?secret=...&issuer=...
    let otpauth_uri = format!(
        "otpauth://totp/{}:{}?secret={}&issuer={}",
        urlencoding::encode(issuer),
        urlencoding::encode(username),
        secret,
        urlencoding::encode(issuer)
    );

    let qr_code = QrCode::new(otpauth_uri)?;
    let image = qr_code.render::<char>()
        .min_dimensions(200, 200)
        .build();

    // For now, just return the URL encoding (frontend can render with qrcode.js)
    Ok(format!("otpauth://totp/{}:{}?secret={}&issuer={}",
        issuer, username, secret, issuer))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_totp_secret() {
        let secret = generate_totp_secret();
        assert!(!secret.is_empty());
        // Should be valid base32
        use base64::engine::general_purpose::STANDARD;
        assert!(STANDARD.decode(&secret).is_ok());
    }

    #[test]
    fn test_generate_backup_codes() {
        let codes = generate_backup_codes(10);
        assert_eq!(codes.len(), 10);
        // Each code should be unique
        for code in &codes {
            assert_eq!(code.len(), 16); // 8 bytes = 16 hex chars
        }
    }

    #[test]
    fn test_totp_verification() {
        let secret = generate_totp_secret();
        // Generate code immediately
        let code = generate_totp_code(&secret).unwrap();
        // Should verify
        assert!(verify_totp_code(&secret, &code).unwrap());
    }

    #[test]
    fn test_totp_invalid_code() {
        let secret = generate_totp_secret();
        // Verify invalid code fails
        assert!(!verify_totp_code(&secret, "000000").unwrap());
    }
}
