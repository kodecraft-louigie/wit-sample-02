wit_bindgen::generate!();

use exports::wasi::http::incoming_handler::Guest as HttpServerTrait;
use wasi::http::types::*;

use wasi::logging::logging::*;

// NOTE: the imports below is generated by wit_bindgen::generate, due to the
// WIT definition(s) specified in `wit`
use wasmcloud::postgres::query::query;
use wasmcloud::postgres::types::{PgValue, ResultRow, ResultRowEntry};

// NOTE: the `Guest` trait corresponds to the export of the `invoke` interface,
// namespaced to the current WIT namespace & package ("wasmcloud:examples")
use exports::wasmcloud::hello::invoke::Guest;

/// This struct must implement the all `export`ed functionality
/// in the WIT definition (see `wit/component.wit`)
struct QueryRunner;

const CREATE_TABLE_QUERY: &str = r#"
CREATE TABLE IF NOT EXISTS example (
  id bigint GENERATED BY DEFAULT AS IDENTITY PRIMARY KEY,
  description text NOT NULL,
  created_at timestamptz NOT NULL DEFAULT NOW()
);
"#;

/// A basic insert query, using Postgres `RETURNING` syntax,
/// which returns the contents of the row that was inserted
const INSERT_QUERY: &str = r#"
INSERT INTO example (description) VALUES ($1) RETURNING *;
"#;

/// A SELECT query, which takes the ID insert query, using Postgres `RETURNING` syntax,
/// which returns the contents of the row that was inserted

// const SELECT_QUERY: &str = r#"
// select ticker from counterparty where id = 35;
// "#;

const SELECT_QUERY: &str = r#"
select currency_pair from spot_prices
WHERE 
	(date_part('year', 	    open_time_date) = 2024)     AND
    (date_part('month', 	open_time_date) = 1) 	AND
    (date_part('day',		open_time_date) >= 1) 	AND
    (date_part('day', 		open_time_date) <= 7)
order by open_time_date asc;
"#;

impl HttpServerTrait for QueryRunner {
    fn handle(_request: IncomingRequest, response_out: ResponseOutparam) {
        let mut resp: String = String::new();
        
        log(Level::Info,"","Retrieving rows...");
        match query(SELECT_QUERY, &[]) {
            Ok(rows) => {
                log(Level::Info,"", format!("Retrieved rows: {:?}", rows.len()).as_str());              
                match rows[0][0].value.clone() {
                    //PgValue::Varchar(value) => log(Level::Info,"",format!("value: {:?}", String::from_utf8(value.1).unwrap()).as_str()),
                    PgValue::Text(value) => {
                        resp = value.clone();
                        log(Level::Info,"", format!("1st row value: {:?}", value).as_str())
                    },
                    _ => log(Level::Info,"","Failed: parse"),
                }                
            },
            Err(e) => {
                log(Level::Info,"", format!("ERROR: failed to retrieve row::{}", e).as_str());
            }
        };

        let response = OutgoingResponse::new(Fields::new());
        response.set_status_code(200).unwrap();       
        let response_body = response.body().unwrap();
        ResponseOutparam::set(response_out, Ok(response));
        response_body
            .write()
            .unwrap()
            .blocking_write_and_flush(format!("Response: {:?}", resp).as_bytes())
            .unwrap();
        OutgoingBody::finish(response_body, None).expect("failed to finish response body");
    }
}

impl Guest for QueryRunner {
    fn call() -> String {
        // First, ensure the right table is present
        if let Err(e) = query(CREATE_TABLE_QUERY, &[]) {
            return format!("ERROR - failed to create table: {e}");
        };

        // Insert a row into the example
        //
        // NOTE: Details of *which* database against which the query is made are
        // managed by the link between this component and it's provider
        let inserted_row: Vec<ResultRow> = match query(
            INSERT_QUERY,
            &[PgValue::Text("inserted example row!".into())],
        ) {
            // We expect to get just one row, since we're searching by ID
            Ok(row) if row.len() == 1 => row,
            // If we get any other number of rows, we error
            Ok(rows) => {
                return format!(
                    "ERROR: unexpected number of rows ({}) returned by SELECT",
                    rows.len()
                );
            }
            // If the query failed, return the error
            Err(e) => {
                return format!("ERROR: failed to insert row: {e}");
            }
        };

        // Extract the "id" column's value (a PgValue) from the result row entries,
        // since ResultRow is a Vec<ResultRowEntry>
        let Some(ResultRowEntry { value: id, .. }) = inserted_row
            .first()
            .and_then(|r| r.iter().find(|entry| entry.column_name == "id"))
        else {
            return "ERROR: returned row is missing an id column".into();
        };

        // Do an explicit SELECT for the row we just inserted, using the ID that was returned
        //
        // NOTE: normally you would not need this SELECT, thanks to RETURNING:
        // https://www.postgresql.org/docs/current/dml-returning.html
        match query(SELECT_QUERY, &[id.clone()]) {
            Ok(rows) => format!("SUCCESS: inserted and manually retrieved new row:\n{rows:#?}"),
            Err(e) => format!("ERROR: failed to retrieve inserted row: {e}"),
        }
    }
}

export!(QueryRunner);