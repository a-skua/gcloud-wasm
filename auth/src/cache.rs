use crate::Token;
use std::cell::RefCell;
use std::collections::HashMap;
use wstd::time::{Duration, Instant};

/// Margin before actual expiry to refresh the token (seconds).
const EXPIRY_MARGIN_SECS: u64 = 60;

struct CachedToken {
    token: Token,
    acquired_at: Instant,
    expires_in_secs: u64,
}

impl CachedToken {
    fn is_valid(&self) -> bool {
        let elapsed = self.acquired_at.elapsed();
        let ttl = Duration::from_secs(self.expires_in_secs.saturating_sub(EXPIRY_MARGIN_SECS));
        elapsed < ttl
    }
}

thread_local! {
    static TOKEN_CACHE: RefCell<HashMap<String, CachedToken>> = RefCell::new(HashMap::new());
}

fn cache_key(scopes: &[String]) -> String {
    let mut sorted = scopes.to_vec();
    sorted.sort();
    sorted.join(" ")
}

pub(crate) fn get(scopes: &[String]) -> Option<Token> {
    let key = cache_key(scopes);
    TOKEN_CACHE.with(|cache| {
        let cache = cache.borrow();
        cache.get(&key).and_then(|cached| {
            if cached.is_valid() {
                Some(Token {
                    access_token: cached.token.access_token.clone(),
                    token_type: cached.token.token_type.clone(),
                    expires_in: cached.token.expires_in,
                })
            } else {
                None
            }
        })
    })
}

pub(crate) fn put(scopes: &[String], token: &Token) {
    let key = cache_key(scopes);
    TOKEN_CACHE.with(|cache| {
        let mut cache = cache.borrow_mut();
        // Remove expired entries while we're at it
        cache.retain(|_, v| v.is_valid());
        cache.insert(
            key,
            CachedToken {
                token: Token {
                    access_token: token.access_token.clone(),
                    token_type: token.token_type.clone(),
                    expires_in: token.expires_in,
                },
                acquired_at: Instant::now(),
                expires_in_secs: token.expires_in,
            },
        );
    });
}
