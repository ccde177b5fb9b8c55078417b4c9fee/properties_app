use std::env;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::path;

use axum::{
    extract::{ContentLengthLimit, Extension, Multipart, Path, Query},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, get_service},
    Json, Router,
};
use axum_extra::routing::SpaRouter;
use serde::Deserialize;
use sqlx::postgres::PgPool;
use tokio::fs::create_dir;
use tower_http::services::fs::ServeDir;
use uuid::Uuid;

mod property;

use property::Property;

#[tokio::main]
async fn main() {
    let upload_dir = path::Path::new("./uploads");
    if !upload_dir.exists() {
        create_dir(upload_dir)
            .await
            .expect("Could not create upload directory");
    }

    let database_url = env::var("DATABASE_URL").expect("No database url in env");
    let pool = PgPool::connect(&database_url)
        .await
        .expect("Could not connect to database");

    let spa = SpaRouter::new("/assets", "dist");
    let v1 = Router::new()
        .route("/property", get(property_list).post(property_create))
        .route(
            "/property/:id",
            get(property_read)
                .post(property_update)
                .delete(property_delete),
        )
        .route("/property/:id/tags", get(tags_list))
        .route(
            "/property/:id/tags/:tag_id",
            get(read_tag_by_id)
                .post(add_tag_by_id)
                .delete(delete_tag_by_id),
        );
    let api = Router::new().nest("/v1", v1).layer(Extension(pool));
    let app = Router::new().merge(spa).nest("/api", api).nest(
        "/uploads",
        get_service(ServeDir::new("uploads")).handle_error(static_serve_error),
    );

    let socket_addr = SocketAddr::from((IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), 8080));
    axum::Server::bind(&socket_addr)
        .serve(app.into_make_service())
        .await
        .expect("Unable to start backend");
}

async fn read_tag_by_id(
    Path((id, tag_id)): Path<(i32, i32)>,
    Extension(pool): Extension<PgPool>,
) -> impl IntoResponse {
    sqlx::query!(
        "SELECT property_id, tag_id 
         FROM property_tags 
         WHERE property_id = $1 AND tag_id = $2",
        id,
        tag_id
    )
    .fetch_optional(&pool)
    .await
    .map_err(|err| (StatusCode::INTERNAL_SERVER_ERROR, format!("{err}")))
    .transpose()
    .ok_or((StatusCode::NOT_FOUND, "Not found"))
    .map(|_| (StatusCode::OK, ""))
}

async fn add_tag_by_id(
    Path((id, tag_id)): Path<(i32, i32)>,
    Extension(pool): Extension<PgPool>,
) -> impl IntoResponse {
    sqlx::query!(
        "INSERT INTO property_tags (property_id, tag_id) 
         VALUES ($1, $2)",
        id,
        tag_id
    )
    .execute(&pool)
    .await
    .map_err(|err| (StatusCode::INTERNAL_SERVER_ERROR, format!("{err}")))
    .map(|_| (StatusCode::OK, "OK".to_string()))
}

async fn delete_tag_by_id(
    Path((id, tag_id)): Path<(i32, i32)>,
    Extension(pool): Extension<PgPool>,
) -> impl IntoResponse {
    sqlx::query!(
        "DELETE FROM property_tags 
         WHERE property_id = $1 AND tag_id = $2",
        id,
        tag_id
    )
    .execute(&pool)
    .await
    .map_err(|err| (StatusCode::INTERNAL_SERVER_ERROR, format!("{err}")))
    .map(|_| (StatusCode::OK, "OK".to_string()))
}

async fn tags_list(Path(id): Path<i32>, Extension(pool): Extension<PgPool>) -> impl IntoResponse {
    sqlx::query!(
        "SELECT tag_id FROM property_tags WHERE property_id = $1",
        id
    )
    .fetch_all(&pool)
    .await
    .map_err(|err| (StatusCode::INTERNAL_SERVER_ERROR, format!("{err}")))
    .map(|vec| Json(vec.iter().map(|anon| anon.tag_id).collect::<Vec<i32>>()))
}

#[derive(Deserialize)]
struct ListQuery {
    page: Option<i64>,
}

async fn property_list(
    Query(query): Query<ListQuery>,
    Extension(pool): Extension<PgPool>,
) -> impl IntoResponse {
    let offset = query.page.map_or(0, |page| page * 20);
    sqlx::query_as!(
        Property,
        "SELECT * FROM properties 
         LIMIT 20 OFFSET $1",
        offset
    )
    .fetch_all(&pool)
    .await
    .map(|properties| Json(properties))
    .map_err(|err| (StatusCode::INTERNAL_SERVER_ERROR, format!("{err}")))
}

async fn property_update(Path(id): Path<u64>) -> impl IntoResponse {
    format!("update property {id}")
}

async fn property_read(
    Path(id): Path<i32>,
    Extension(pool): Extension<PgPool>,
) -> impl IntoResponse {
    sqlx::query_as!(Property, "SELECT * FROM properties WHERE id = $1", id)
        .fetch_one(&pool)
        .await
        .map(|property| Json(property))
        .map_err(|err| (StatusCode::INTERNAL_SERVER_ERROR, format!("{err}")))
}

async fn download_multipart_fields(
    multipart: &mut Multipart,
    upload_dir: &path::PathBuf,
) -> Result<(), (StatusCode, String)> {
    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|err| (StatusCode::BAD_REQUEST, format!("{err}")))?
    {
        let upload_location = field
            .file_name()
            .map(|filename| upload_dir.join(filename))
            .ok_or((StatusCode::BAD_REQUEST, "Empty filename".to_string()))?;
        let data = field
            .bytes()
            .await
            .map_err(|err| (StatusCode::BAD_REQUEST, format!("{err}")))?;
        tokio::fs::write(upload_location, data)
            .await
            .map_err(|err| (StatusCode::INTERNAL_SERVER_ERROR, format!("{err}")))?;
    }
    Ok(())
}

async fn property_create(
    Extension(pool): Extension<PgPool>,
    ContentLengthLimit(mut multipart): ContentLengthLimit<Multipart, { 1024 * 1024 * 100 }>,
) -> Result<Json<i32>, (StatusCode, String)> {
    let field = multipart
        .next_field()
        .await
        .map_err(|err| (StatusCode::BAD_REQUEST, format!("{err}")))?
        .ok_or((StatusCode::BAD_REQUEST, "Multipart is empty".to_string()))?;

    field
        .content_type()
        .filter(|content_type| content_type.contains("application/json"))
        .ok_or((
            StatusCode::BAD_REQUEST,
            format!("Wrong content type {}", field.content_type().unwrap()),
        ))?;

    let property: Property = field
        .text()
        .await
        .map_err(|err| (StatusCode::BAD_REQUEST, format!("{err}")))
        .and_then(|text| {
            serde_json::from_str(&text).map_err(|err| (StatusCode::BAD_REQUEST, format!("{err}")))
        })?;

    let uuid = Uuid::new_v4().to_string();
    let upload_dir = path::Path::new("uploads").join(&uuid);
    create_dir(&upload_dir)
        .await
        .map_err(|err| (StatusCode::INTERNAL_SERVER_ERROR, format!("{err}")))?;
    let download_result = download_multipart_fields(&mut multipart, &upload_dir).await;
    if download_result.is_err() {
        tokio::fs::remove_dir_all(upload_dir).await;
        return Err(download_result.unwrap_err());
    }

    let insert_result = sqlx::query!(
        "INSERT INTO properties 
        (name, location, area, property_type, wc, floor, tothesea, furniture, appliances, price, gallery_location) 
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11) 
        RETURNING id",
        property.name,
        property.location,
        property.area,
        property.property_type,
        property.wc,
        property.floor,
        property.tothesea,
        property.furniture,
        property.appliances,
        property.price,
        uuid).fetch_one(&pool).await.map(|result| Json(result.id)).map_err(|err| (StatusCode::INTERNAL_SERVER_ERROR, format!("{err}")));
    if insert_result.is_err() {
        tokio::fs::remove_dir_all(upload_dir).await;
    }
    insert_result
}

async fn property_delete(
    Extension(pool): Extension<PgPool>,
    Path(id): Path<i32>,
) -> impl IntoResponse {
    sqlx::query!("DELETE FROM properties WHERE id = $1", id)
        .execute(&pool)
        .await
        .map(|_| (StatusCode::OK, ""))
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, ""))
}

async fn static_serve_error(_err: std::io::Error) -> impl IntoResponse {
    (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error")
}
