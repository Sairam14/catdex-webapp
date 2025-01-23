mod models;
use self::models::*;
mod schema;
use self::schema::cats::dsl::*;
use actix_files::Files;
use actix_web::{error::ErrorInternalServerError, web, App, Error, HttpResponse, HttpServer, http::header};
use awmp::Parts;
use diesel::pg::PgConnection;
use diesel::prelude::*;
use diesel::r2d2::{self, ConnectionManager};
use handlebars::Handlebars;
use std::env;
use std::collections::HashMap;

// Define the type alias for the connection pool
pub type DbPool = r2d2::Pool<ConnectionManager<PgConnection>>;
pub type CatsData = Result<Vec<Cat>, diesel::result::Error>;

async fn add(data: web::Data<AppState>) -> Result<HttpResponse, Error> {
    let body = data.hb.render("add", &{}).unwrap();
    Ok(HttpResponse::Ok().body(body))
}

async fn add_cat_form(data: web::Data<AppState>, mut parts: Parts) -> Result<HttpResponse, Error> {
    let file_path = parts
        .files
        .take("image")
        .pop()
        .and_then(|f| f.persist_in("./static/image").ok())
        .unwrap_or_default();
    
    let text_fields: HashMap<_,_> = parts.texts.as_pairs().into_iter().collect();
    let mut connection = data.pool.get().expect("Can't get db connection from pool");

    let new_cat = NewCat {
        name: text_fields.get("name").unwrap().to_string(),
        image_path: file_path.to_str().unwrap().to_string(),
    };
    web::block(move || {
        diesel::insert_into(cats)
            .values(&new_cat)
            .execute(&mut *connection)
    })
    .await?
    .map_err(|e| {
        eprintln!("Database error: {}", e);
        ErrorInternalServerError(e)
    })?;

    Ok(HttpResponse::SeeOther()
        .append_header((header::LOCATION, "/"))
        .finish())
}

async fn index(app_data: web::Data<AppState>) -> Result<HttpResponse, Error> {
    let mut connection = app_data.pool.get().expect("Can't get db connection from pool");
    let cats_data: Vec<Cat> = web::block(move || cats.limit(100).load::<Cat>(&mut *connection))
        .await?
        .map_err(|e| {
            eprintln!("Database error: {}", e);
            ErrorInternalServerError(e)
        })?;
    let data = IndexTemplateData {
        project_name: "Catdex".to_string(),
        cats: cats_data,
    };
    let body = app_data.hb.render("index", &data).unwrap();
    Ok(HttpResponse::Ok().body(body))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();
    // Setting up the handlebar template engine
    let mut handlebars = Handlebars::new();
    handlebars
        .register_templates_directory(".html", "./static/")
        .unwrap();
    let handlebars_ref = handlebars;
    // Setting up the database connection pool
    let database_url = match env::var("DATABASE_URL") {
        Ok(url) => url,
        Err(_) => "postgres://postgres:mypassword@localhost".to_string(),
    };
    let manager = ConnectionManager::<PgConnection>::new(&database_url);
    let pool = r2d2::Pool::builder()
        .build(manager)
        .expect("Failed to create DB connection pool.");
    println!("Listening on port 8080");
    let app_state = AppState {
        hb: handlebars_ref,
        pool: pool.clone(),
    };
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(app_state.clone()))
            .service(Files::new("/static", "static").show_files_listing())
            .route("/", web::get().to(index))
            .route("/add", web::get().to(add))
            .route("/add_cat_form", web::post().to(add_cat_form))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}

#[derive(Clone)]
struct AppState {
    hb: Handlebars<'static>,
    pool: DbPool,
}
