use async_trait::async_trait;

#[async_trait]
trait KeyValue {
    async fn get(&self, key: String) -> Option<String>;

    async fn set(&self, key: String, value: String) -> ();
}
