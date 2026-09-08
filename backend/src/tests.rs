#[cfg(test)]
mod auth_tests {
    use crate::services::auth_service::{hash_token, validate_password, generate_media_token, validate_media_token};

    #[test]
    fn test_hash_token_sha256() {
        let token = "test-token-12345";
        let hash = hash_token(token);
        
        // SHA-256 должен давать 64 hex символа
        assert_eq!(hash.len(), 64);
        
        // Один и тот же токен должен давать одинаковый хэш
        assert_eq!(hash, hash_token(token));
        
        // Разные токены должны давать разные хэши
        assert_ne!(hash, hash_token("different-token"));
    }

    #[test]
    fn test_validate_password_length() {
        let result = validate_password("short", 8);
        assert!(result.is_err());
        
        let result = validate_password("longenoughpassword", 8);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_password_common() {
        let result = validate_password("password", 8);
        assert!(result.is_err());
        
        let result = validate_password("12345678", 8);
        assert!(result.is_err());
        
        let result = validate_password("MyUn1queP@ss!", 8);
        assert!(result.is_ok());
    }
}

#[cfg(test)]
mod streaming_tests {
    use crate::services::streaming_service::{generate_media_token, validate_media_token};

    #[test]
    fn test_generate_and_validate_media_token() {
        let secret = "test-secret-key-for-testing-only";
        let user_id = uuid::Uuid::new_v4();
        let content_id = uuid::Uuid::new_v4();
        
        let token = generate_media_token(
            user_id,
            content_id,
            true,
            120,
            secret,
        ).unwrap();
        
        let claims = validate_media_token(&token, secret).unwrap();
        
        assert_eq!(claims.user_id, user_id);
        assert_eq!(claims.content_id, content_id);
        assert!(claims.is_full_access);
    }

    #[test]
    fn test_media_token_wrong_secret() {
        let secret_correct = "correct-secret-key-for-testing";
        let secret_wrong = "wrong-secret-key-for-testing!!";
        let user_id = uuid::Uuid::new_v4();
        let content_id = uuid::Uuid::new_v4();
        
        let token = generate_media_token(
            user_id,
            content_id,
            true,
            120,
            secret_correct,
        ).unwrap();
        
        let result = validate_media_token(&token, secret_wrong);
        assert!(result.is_err());
    }
}

#[cfg(test)]
mod payment_tests {
    use crate::services::payment_service::verify_yookassa_webhook;
    use hmac::{Hmac, Mac};
    use sha2::Sha256;

    type HmacSha256 = Hmac<Sha256>;

    #[test]
    fn test_webhook_verification_valid() {
        let secret = "test-webhook-secret";
        let body = r#"{"event":"payment.succeeded","object":{"id":"test-123"}}"#;
        
        // Генерируем правильную подпись
        let mut mac = HmacSha256::new_from_slice(secret.as_bytes()).unwrap();
        mac.update(body.as_bytes());
        let signature = hex::encode(mac.finalize().into_bytes());
        
        let result = verify_yookassa_webhook(body, &signature, secret);
        assert!(result.is_ok());
    }

    #[test]
    fn test_webhook_verification_invalid() {
        let secret = "test-webhook-secret";
        let body = r#"{"event":"payment.succeeded","object":{"id":"test-123"}}"#;
        let wrong_signature = "0000000000000000000000000000000000000000000000000000000000000000";
        
        let result = verify_yookassa_webhook(body, wrong_signature, secret);
        assert!(result.is_err());
    }
}
