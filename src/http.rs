pub type HttpResult<T> = Result<T, rocket::response::Debug<anyhow::Error>>;
