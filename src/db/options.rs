pub trait PoolConnection {
    type Pool;

    fn get() -> Self::Pool;
    fn init() -> impl std::future::Future<Output = ()> + Send;
}
