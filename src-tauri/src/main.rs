// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

// Learn more about Tauri commands at https://tauri.app/v1/guides/features/command

use duckdb::arrow::util::display::{ArrayFormatter, FormatOptions};
use duckdb::Error as DuckDBError;
use duckdb::{arrow::array::RecordBatch, Connection};
use ignore::WalkBuilder;
use serde::Serialize;
use std::env;
use std::fs;
use std::path::Path;

#[derive(Debug, thiserror::Error, Serialize)]
enum QueryError {
    #[error("DuckDB error: {0}")]
    DuckDB(String),
    #[error("No results returned")]
    NoResults,
}

impl From<DuckDBError> for QueryError {
    fn from(err: DuckDBError) -> Self {
        QueryError::DuckDB(err.to_string())
    }
}

#[tauri::command]
fn create_table(query: &str) -> Result<String, String> {
    println!("{:?}", &query);

    let binding = Connection::open_in_memory().map_err(|e| e.to_string())?;

    let mut stmt = binding.prepare(query).map_err(|e| e.to_string())?;

    let results: Vec<RecordBatch> = stmt.query_arrow([]).map_err(|e| e.to_string())?.collect();

    if results.is_empty() {
        return Err(QueryError::NoResults.to_string());
    }

    let options = FormatOptions::default();

    let headers: Vec<String> = results[0]
        .schema()
        .fields
        .iter()
        .map(|f| f.name().clone())
        .collect();

    let rows: Vec<Vec<String>> = (0..results[0].num_rows())
        .map(|row| {
            results[0]
                .columns()
                .iter()
                .map(|c| {
                    let formatter = ArrayFormatter::try_new(c.as_ref(), &options).unwrap();
                    formatter.value(row).to_string()
                })
                .collect()
        })
        .collect();

    let headers_html = headers
        .iter()
        .map(|h| format!("<th>{}</th>", h))
        .collect::<Vec<String>>()
        .join("");

    let rows_html = rows
        .iter()
        .map(|r| {
            format!(
                "<tr>{}</tr>",
                r.iter()
                    .map(|c| format!("<td>{}</td>", c))
                    .collect::<Vec<String>>()
                    .join("")
            )
        })
        .collect::<Vec<String>>()
        .join("");

    Ok(format!(
        r#"<table id="table" class="table table-md table-pin-rows"><thead><tr>{}</tr></thead><tbody>{}</tbody></table>"#,
        headers_html, rows_html
    ))
}

#[tauri::command]
fn list_files(directory: Option<String>) -> Result<String, String> {
    fn list_files_recursive(path: &Path, depth: usize) -> Result<String, String> {
        let mut html = String::new();
        let entries = fs::read_dir(path).map_err(|e| e.to_string())?;

        for entry in entries {
            let entry = entry.map_err(|e| e.to_string())?;
            let path = entry.path();

            if path.is_dir() {
                let dir_content = list_files_recursive(&path, depth + 1)?;
                if !dir_content.is_empty() {
                    html.push_str(&format!(
                        r#"<li>
                            <details>
                                <summary>
                                    <svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M3 7v10a2 2 0 002 2h14a2 2 0 002-2V9a2 2 0 00-2-2h-6l-2-2H5a2 2 0 00-2 2z" />
                                    </svg>
                                    {0}
                                </summary>
                                <ul class="menu ml-4">{1}</ul>
                            </details>
                        </li>"#,
                        path.file_name().unwrap_or_default().to_string_lossy(),
                        dir_content
                    ));
                }
            } else if let Some(extension) = path.extension() {
                let ext = extension.to_string_lossy().to_lowercase();
                if ext == "sql" {
                    html.push_str(&format!(
                        r#"<li>
                            <a class="sql-file" data-path="{0}">
                                <svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z" />
                                </svg>
                                {1}
                            </a>
                        </li>"#,
                        path.to_string_lossy(),
                        path.file_name().unwrap().to_string_lossy()
                    ));
                } else if ["csv", "xlsx", "parquet"].contains(&ext.as_str()) {
                    html.push_str(&format!(
                        r#"<li>
                            <a class="data-file">
                                <svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 7v10c0 2 1 3 3 3h10c2 0 3-1 3-3V7c0-2-1-3-3-3H7c-2 0-3 1-3 3z" />
                                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 17v-6" />
                                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 17v-3" />
                                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 17v-5" />
                                </svg>
                                {0}
                            </a>
                        </li>"#,
                        path.file_name().unwrap().to_string_lossy()
                    ));
                }
            }
        }

        Ok(html)
    }

    let path = directory
        .map(|d| Path::new(&d).to_path_buf())
        .unwrap_or_else(|| env::current_dir().unwrap());

    let html = format!(
        r#"<li>
            <details open>
                <summary>
                    <svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M3 7v10a2 2 0 002 2h14a2 2 0 002-2V9a2 2 0 00-2-2h-6l-2-2H5a2 2 0 00-2 2z" />
                    </svg>
                    {0}
                </summary>
                <ul class="menu">{1}</ul>
            </details>
        </li>"#,
        path.file_name().unwrap_or_default().to_string_lossy(),
        list_files_recursive(&path, 0)?
    );

    Ok(html)
}

#[tauri::command]
fn read_file(path: &str) -> Result<String, String> {
    fs::read_to_string(path).map_err(|e| e.to_string())
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            create_table,
            list_files,
            read_file
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
