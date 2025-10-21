use uuid::Uuid;

/// Create a cryptographically secure session token
pub fn create_session_token() -> String {
    // Use UUID v4 as session token (128-bit random value)
    Uuid::new_v4().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_session_token_uniqueness() {
        let token1 = create_session_token();
        let token2 = create_session_token();
        assert_ne!(token1, token2);
    }

    #[test]
    fn test_session_token_format() {
        let token = create_session_token();
        // UUID format: 8-4-4-4-12 hex digits
        assert_eq!(token.len(), 36); // UUID string representation length
    }
}
