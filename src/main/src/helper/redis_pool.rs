pub type RedisPool = r2d2::Pool<redis::Client>;
pub type RedisConnection = r2d2::PooledConnection<redis::Client>;

pub fn create_redis_pool(url: &str) -> Option<RedisPool> {
    let Ok(client) = redis::Client::open(url) else {
        return None;
    };
    //TODO: configure pool
    r2d2::Pool::new(client).ok()
}
