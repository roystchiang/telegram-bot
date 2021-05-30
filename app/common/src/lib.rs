use async_trait::async_trait;
use hyper::{Body, Method, Request, Response};

#[async_trait]
pub trait Server {
    async fn serve(
        &self,
        request: Request<Body>,
    ) -> Result<Response<Body>, Box<dyn std::error::Error + Send + Sync>>;
}

pub struct CommonServer<T>
where
    T: Server + Send + Sync,
{
    server: T,
}

#[async_trait]
impl<T: Server + Send + Sync> Server for CommonServer<T> {
    async fn serve(
        &self,
        request: Request<Body>,
    ) -> Result<Response<Body>, Box<dyn std::error::Error + Send + Sync>> {
        match (request.method(), request.uri().path()) {
            (&Method::GET, "/health") => Ok(Response::new("healthy".into())),
            _ => {
                let result = self.server.serve(request);
                return result.await;
            }
        }
    }
}

impl<T: Server + Send + Sync> CommonServer<T> {
    pub fn new(server: T) -> Self {
        Self { server }
    }
}

#[cfg(test)]
mod test {
    use async_trait::async_trait;
    use hyper::{body::to_bytes, Body, Request, Response, StatusCode};

    use crate::{CommonServer, Server};

    struct TestServer;

    #[async_trait]
    impl Server for TestServer {
        async fn serve(
            &self,
            _request: Request<Body>,
        ) -> Result<Response<Body>, Box<dyn std::error::Error + Send + Sync>> {
            Ok(Response::new("test".into()))
        }
    }

    #[tokio::test]
    async fn should_route_health_endpoint() {
        let server = CommonServer::new(TestServer {});
        let request = Request::builder().uri("/health").body("".into()).unwrap();

        let result = server.serve(request).await.unwrap();

        assert_eq!(result.status(), StatusCode::OK);
        assert_eq!(to_bytes(result).await.unwrap(), "healthy");
    }

    #[tokio::test]
    async fn should_route_custom_server_endpoint() {
        let server = CommonServer::new(TestServer {});
        let request = Request::builder()
            .uri("/some/path")
            .body("".into())
            .unwrap();

        let result = server.serve(request).await.unwrap();

        assert_eq!(result.status(), StatusCode::OK);
        assert_eq!(to_bytes(result).await.unwrap(), "test");
    }
}
