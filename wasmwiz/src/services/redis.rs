// src/services/redis.rs
use anyhow::Result;
use redis::{Client, AsyncCommands};
use std::sync::Arc;

#[derive(Clone)]
pub struct RedisService {
    client: Arc<Client>,
}

impl RedisService {
    pub fn new(redis_url: &str) -> Result<Self> {
        let client = Client::open(redis_url)?;
        Ok(Self {
            client: Arc::new(client),
        })
    }

    pub async fn get_connection(&self) -> Result<redis::aio::Connection> {
        Ok(self.client.get_async_connection().await?)
    }

    /// Set a key with expiration in seconds
    pub async fn set_ex(&self, key: &str, value: &str, expiry_secs: usize) -> Result<()> {
        let mut conn = self.get_connection().await?;
        conn.set_ex(key, value, expiry_secs).await?;
        Ok(())
    }

    /// Get a key's value
    pub async fn get(&self, key: &str) -> Result<Option<String>> {
        let mut conn = self.get_connection().await?;
        let result: Option<String> = conn.get(key).await?;
        Ok(result)
    }

    /// Increment a key and return the new value
    pub async fn incr(&self, key: &str) -> Result<i64> {
        let mut conn = self.get_connection().await?;
        let value: i64 = conn.incr(key, 1).await?;
        Ok(value)
    }

    /// Set expiration on a key
    pub async fn expire(&self, key: &str, seconds: usize) -> Result<bool> {
        let mut conn = self.get_connection().await?;
        let result: bool = conn.expire(key, seconds).await?;
        Ok(result)
    }

    /// Atomic: Increment a key and set expiration if it doesn't exist
    pub async fn incr_and_expire(&self, key: &str, seconds: usize) -> Result<i64> {
        let script = r#"
            local count = redis.call('INCR', KEYS[1])
            if count == 1 then
                redis.call('EXPIRE', KEYS[1], ARGV[1])
            end
            return count
        "#;
        
        let mut conn = self.get_connection().await?;
        let result: i64 = redis::Script::new(script)
            .key(key)
            .arg(seconds)
            .invoke_async(&mut conn)
            .await?;
        
        Ok(result)
    }

    /// Get the time-to-live for a key in seconds
    pub async fn ttl(&self, key: &str) -> Result<i64> {
        let mut conn = self.get_connection().await?;
        let ttl: i64 = conn.ttl(key).await?;
        Ok(ttl)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::time::{sleep, Duration};

    #[tokio::test]
    async fn test_redis_operations() {
        // Skip test if REDIS_URL env var is not set
        let redis_url = match std::env::var("REDIS_URL") {
            Ok(url) => url,
            Err(_) => {
                println!("Skipping Redis test, REDIS_URL not set");
                return;
            }
        };

        let service = RedisService::new(&redis_url).expect("Failed to connect to Redis");
        
        // Test set and get
        let test_key = "test_key_1";
        service.set_ex(test_key, "test_value", 10).await.unwrap();
        let value = service.get(test_key).await.unwrap();
        assert_eq!(value, Some("test_value".to_string()));
        
        // Test incr
        let counter_key = "test_counter_1";
        let count1 = service.incr(counter_key).await.unwrap();
        let count2 = service.incr(counter_key).await.unwrap();
        assert_eq!(count1, 1);
        assert_eq!(count2, 2);
        
        // Test expire
        service.expire(counter_key, 1).await.unwrap();
        sleep(Duration::from_secs(2)).await;
        let value = service.get(counter_key).await.unwrap();
        assert_eq!(value, None);
        
        // Test incr_and_expire
        let auto_key = "test_auto_expire_1";
        let count = service.incr_and_expire(auto_key, 1).await.unwrap();
        assert_eq!(count, 1);
        let ttl = service.ttl(auto_key).await.unwrap();
        assert!(ttl > 0);
        sleep(Duration::from_secs(2)).await;
        let value = service.get(auto_key).await.unwrap();
        assert_eq!(value, None);
    }
}
