use axum::{
    RequestPartsExt, Router,
    extract::{OriginalUri, Request},
    response::IntoResponse,
    routing::get,
};
use http::StatusCode;
use tower::ServiceExt;
use tower_http::{
    compression::CompressionLayer,
    services::{ServeDir, ServeFile},
};

pub fn serve_spa<S>(path: &str) -> Router<S>
where
    S: Clone + Send + Sync + 'static,
{
    let index_file_path = format!("./dist/{}/index.html", path);

    Router::new()
        // Serve `/path`
        .route(
            &format!("/{path}"),
            get({
                let index_file_path = index_file_path.clone();
                move |req: Request<axum::body::Body>| async move {
                    let serve_file = ServeFile::new(index_file_path.clone());
                    serve_file.oneshot(req).await.into_response()
                }
            }),
        )
        // Serve `/path/{*route}`
        .route(
            &format!("/{path}/{{*route}}"),
            get({
                let index_file_path = index_file_path.clone();
                move |req: Request<axum::body::Body>| async move {
                    let serve_file = ServeFile::new(index_file_path.clone());
                    serve_file.oneshot(req).await.into_response()
                }
            }),
        )
}

pub fn serve_static_files<S>() -> Router<S>
where
    S: Clone + Send + Sync + 'static,
{
    let public_path = "./dist";
    let fallback_service = ServeDir::new(public_path)
        .append_index_html_on_directories(true)
        .not_found_service(ServeFile::new(format!("{}/{}", public_path, "404.html")));
    let compression_layer: CompressionLayer = CompressionLayer::new().gzip(true);

    // Base router
    Router::new()
        .fallback(get(|req: Request| async move {
            let (mut parts, body) = req.into_parts();
            let uri: OriginalUri = parts.extract().await?;

            tracing::info!("Request path: {}", uri.path());
            let req = Request::from_parts(parts, body);
            match fallback_service.oneshot(req).await {
                Ok(mut res) => match res.status() {
                    StatusCode::OK => {
                        if uri.path().contains("/_static/") {
                            res.headers_mut().insert(
                                "Cache-Control",
                                // One year cache
                                "public, max-age=31536000".parse().unwrap(),
                            );
                        }
                        if uri.path().contains("/_astro/") {
                            res.headers_mut().insert(
                                "Cache-Control",
                                // One month cache
                                "public, max-age=2628000".parse().unwrap(),
                            );
                        }
                        Ok(res)
                    }
                    _ => Ok(res),
                },
                Err(e) => {
                    tracing::error!("fallback_service error: {}", e);
                    Err(e)
                }
            }
        }))
        .layer(compression_layer)
}
