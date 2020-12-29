use actix_web::{get, http::header, post, web, App, HttpResponse, HttpServer, ResponseError};
use askama::Template;
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use rusqlite::params;
use serde::Deserialize;
use thiserror::Error;

#[derive(Deserialize)]
struct AddParams {
    text: String,
    flag: String,
}

#[derive(Deserialize)]
struct AnswerParams {
    id: u32,
    flag: String,
}

// text -> flag? or explain?
struct QuestionEntry {
    id: u32,
    text: String,
}

#[derive(Template)]
#[template(path = "index.html", print ="all")]
struct IndexTemplate {
    entries: Vec<QuestionEntry>,
}

#[derive(Error, Debug)]
enum MyError {
    #[error("failed to render HTML")]
    AskamaError(#[from] askama::Error),

    #[error("failed to get connection")]
    ConnectionPoolError(#[from] r2d2::Error),

    #[error("failed SQL execution")]
    SQLiteError(#[from] rusqlite::Error),
}

impl ResponseError for MyError {}

// ----------

#[post("/add")]
async fn add_question(
    params: web::Form<AddParams>,
    db: web::Data<r2d2::Pool<SqliteConnectionManager>>,
) -> Result<HttpResponse, MyError> {
    dbg!("This is debug");
    let conn = db.get()?;
    conn.execute("INSERT INTO question (text, flag) VALUES (?1, ?2)", &[&params.text, &params.flag])?;
    Ok(HttpResponse::SeeOther()
        .header(header::LOCATION, "/")
        .finish())
}

#[post("/answer")]
async fn answer_question(
    params: web::Form<AnswerParams>,
    db: web::Data<r2d2::Pool<SqliteConnectionManager>>,
) -> Result<HttpResponse, MyError> {
    let conn = db.get()?;
    // Todo
    dbg!("Before prepare statement");

    let mut cached_statement = conn.prepare_cached(
        "SELECT id, text, flag FROM question WHERE id = ?1 AND flag = ?2")?;

    let rows = cached_statement.query_map(
        &[&params.id.to_string(), &params.flag.to_string()],
        |_row| {Ok(1)}
    )?;

    if rows.count() > 0 {
         dbg!("Correct!");
    } else {
        dbg!("Incorrect.");
    }

    // Do nothing
    Ok(HttpResponse::SeeOther()
        .header(header::LOCATION, "/")
        .finish())
}


#[get("/")]
async fn index(db: web::Data<Pool<SqliteConnectionManager>>) -> Result<HttpResponse, MyError> {
    // DB接続
    let conn = db.get()?;
    let mut statement = conn.prepare("SELECT id, text FROM question")?;
    // DBクエリ結果をrowsに収納。ラベル付けも同時にする。
    let rows = statement.query_map(params![], |row| {
        let id = row.get(0)?;
        let text = row.get(1)?;
        Ok(QuestionEntry {id, text})
    })?;

    let mut entries = Vec::new();
    for row in rows {
        entries.push(row?);
    }

    // htmlテンプレートにentriesを渡す。
    let html = IndexTemplate { entries };
    let response_body = html.render()?;
    Ok(HttpResponse::Ok()
        .content_type("text/html")
        .body(response_body))
}

#[actix_rt::main]
async fn main() -> Result<(), actix_web::Error> {
    let manager = SqliteConnectionManager::file("srictf.db");
    let pool = Pool::new(manager).expect("Failed to initialize the connection pool.");
    let conn = pool
        .get()
        .expect("Failed to get the connection from the pool");

    conn.execute(
        "CREATE TABLE IF NOT EXISTS question (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            text TEXT NOT NULL,
            flag TEXT NOT NULL
        )",
        params![],)
        .expect("Failed to create a table `question`.");

    println!("Start SRICTF");

    HttpServer::new(move || {
        App::new()
            .service(index)
            .service(add_question)
            .service(answer_question)
            .data(pool.clone())
        })
        .bind("0.0.0.0:8080")?
        .run()
        .await?;
    Ok(())
}
