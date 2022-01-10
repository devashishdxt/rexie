//! Test suite for the Web and headless browsers.

#![cfg(target_arch = "wasm32")]

extern crate wasm_bindgen_test;

use std::{assert, assert_eq, option::Option};

use rexie::{Index, KeyRange, ObjectStore, Result, Rexie, TransactionMode};
use serde::{Deserialize, Serialize};
use wasm_bindgen::JsValue;
use wasm_bindgen_test::*;

wasm_bindgen_test_configure!(run_in_browser);

#[derive(Debug, Deserialize)]
struct Employee {
    id: u32,
    name: String,
    email: String,
}

#[derive(Debug, Serialize)]
struct EmployeeRequest<'a> {
    name: &'a str,
    email: &'a str,
}

/// Creates a database
async fn create_db() -> Rexie {
    assert!(Rexie::delete("test").await.is_ok());

    let rexie = Rexie::builder("test")
        .version(1)
        .add_object_store(
            ObjectStore::new("employees")
                .key_path("id")
                .auto_increment(true)
                .add_index(Index::new("email", "email").unique(true)),
        )
        .add_object_store(ObjectStore::new("departments").auto_increment(true))
        .build()
        .await;
    assert!(rexie.is_ok());
    rexie.unwrap()
}

/// Checks basic details of the database
async fn basic_test_db(rexie: &Rexie) {
    assert_eq!(rexie.name(), "test");
    assert_eq!(rexie.version(), 1.0);
    assert_eq!(rexie.store_names(), vec!["departments", "employees"]);

    let transaction = rexie.transaction(&["employees"], TransactionMode::ReadOnly);
    assert!(transaction.is_ok());
    let transaction = transaction.unwrap();

    assert_eq!(transaction.mode(), TransactionMode::ReadOnly);
    assert_eq!(transaction.store_names(), vec!["employees"]);

    let employees = transaction.store("employees");
    assert!(employees.is_ok());
    let employees = employees.unwrap();

    assert_eq!(employees.name(), "employees");
    assert!(employees.auto_increment());
    assert_eq!(employees.key_path(), Ok(Some("id".to_string())));
    assert_eq!(employees.index_names(), vec!["email"]);

    let email_index = employees.index("email");
    assert!(email_index.is_ok());
    let email_index = email_index.unwrap();

    assert_eq!(email_index.name(), "email");
    assert!(email_index.unique());
    assert!(!email_index.multi_entry());

    assert!(transaction.commit().await.is_ok());
}

/// Closes and deletes the database
async fn close_and_delete_db(rexie: Rexie) {
    rexie.close();
    assert!(Rexie::delete("test").await.is_ok());
}

async fn add_employee(rexie: &Rexie, name: &str, email: &str) -> Result<u32> {
    let transaction = rexie.transaction(&["employees"], TransactionMode::ReadWrite);
    assert!(transaction.is_ok());
    let transaction = transaction.unwrap();

    let employees = transaction.store("employees");
    assert!(employees.is_ok());
    let employees = employees.unwrap();

    let employee = EmployeeRequest { name, email };
    let employee = serde_wasm_bindgen::to_value(&employee).unwrap();
    let employee_id = employees.add(&employee, None).await?;

    transaction.commit().await?;
    Ok(num_traits::cast(employee_id.as_f64().unwrap()).unwrap())
}

async fn get_employee(rexie: &Rexie, id: u32) -> Result<Option<Employee>> {
    let transaction = rexie.transaction(&["employees"], TransactionMode::ReadOnly);
    assert!(transaction.is_ok());
    let transaction = transaction.unwrap();

    let employees = transaction.store("employees");
    assert!(employees.is_ok());
    let employees = employees.unwrap();

    let employee = employees.get(&id.into()).await?;
    let employee: Option<Employee> = serde_wasm_bindgen::from_value(employee).unwrap();

    Ok(employee)
}

async fn get_all_employees(rexie: &Rexie) -> Result<Vec<Employee>> {
    let transaction = rexie.transaction(&["employees"], TransactionMode::ReadOnly);
    assert!(transaction.is_ok());
    let transaction = transaction.unwrap();

    let employees = transaction.store("employees");
    assert!(employees.is_ok());
    let employees = employees.unwrap();

    let employees: Vec<JsValue> = employees
        .get_all(None, None, None)
        .await?
        .into_iter()
        .map(|pair| pair.1)
        .collect();
    let employees: Vec<Employee> = employees
        .into_iter()
        .map(|employee| serde_wasm_bindgen::from_value(employee).unwrap())
        .collect();

    Ok(employees)
}

async fn count_employees(rexie: &Rexie, key_range: Option<&KeyRange>) -> Result<u32> {
    let transaction = rexie.transaction(&["employees"], TransactionMode::ReadOnly);
    assert!(transaction.is_ok());
    let transaction = transaction.unwrap();

    let employees = transaction.store("employees");
    assert!(employees.is_ok());
    let employees = employees.unwrap();

    employees.count(key_range).await
}

async fn clear_employees(rexie: &Rexie) -> Result<()> {
    let transaction = rexie.transaction(&["employees"], TransactionMode::ReadWrite);
    assert!(transaction.is_ok());
    let transaction = transaction.unwrap();

    let employees = transaction.store("employees");
    assert!(employees.is_ok());
    let employees = employees.unwrap();

    employees.clear().await
}

#[wasm_bindgen_test]
async fn test_db_creation_pass() {
    let rexie = create_db().await;
    basic_test_db(&rexie).await;
    close_and_delete_db(rexie).await;
}

#[wasm_bindgen_test]
async fn test_db_add_pass() {
    let rexie = create_db().await;

    // Write values to the database.
    let id = add_employee(&rexie, "John Doe", "john@example.com").await;
    assert_eq!(id, Ok(1));

    let id2 = add_employee(&rexie, "Scooby Doo", "scooby@example.com").await;
    assert_eq!(id2, Ok(2));

    // Read the values back from the database.

    let employee = get_employee(&rexie, 1).await;
    assert!(employee.is_ok());
    let employee = employee.unwrap();
    assert!(employee.is_some());
    let employee = employee.unwrap();

    assert_eq!(employee.id, 1);
    assert_eq!(employee.name, "John Doe");
    assert_eq!(employee.email, "john@example.com");

    let employee = get_employee(&rexie, 2).await;
    assert!(employee.is_ok());
    let employee = employee.unwrap();
    assert!(employee.is_some());
    let employee = employee.unwrap();

    assert_eq!(employee.id, 2);
    assert_eq!(employee.name, "Scooby Doo");
    assert_eq!(employee.email, "scooby@example.com");

    let employee = get_employee(&rexie, 3).await;
    assert!(employee.is_ok());
    let employee = employee.unwrap();
    assert!(employee.is_none());

    close_and_delete_db(rexie).await;
}

#[wasm_bindgen_test]
async fn test_db_duplicate_add_fail() {
    let rexie = create_db().await;

    // Write a value to the database.
    let id = add_employee(&rexie, "John Doe", "john@example.com").await;
    assert_eq!(id, Ok(1));

    // Write a duplicate value (with same email) to the database.
    let id = add_employee(&rexie, "John Doe New", "john@example.com").await;
    assert!(id.is_err());
    let err = id.unwrap_err();
    assert!(err
        .to_string()
        .starts_with("failed to execute indexed db request: ConstraintError"));

    close_and_delete_db(rexie).await;
}

#[wasm_bindgen_test]
async fn test_db_count_and_clear_pass() {
    let rexie = create_db().await;

    // Write values to the database.
    let id = add_employee(&rexie, "John Doe", "john@example.com").await;
    assert_eq!(id, Ok(1));

    let id2 = add_employee(&rexie, "Scooby Doo", "scooby@example.com").await;
    assert_eq!(id2, Ok(2));

    // Count the number of values in the database before and after clearing.
    assert_eq!(count_employees(&rexie, None).await, Ok(2));
    assert_eq!(
        count_employees(&rexie, Some(&KeyRange::only(&1u32.into()).unwrap())).await,
        Ok(1)
    );
    assert_eq!(
        count_employees(
            &rexie,
            Some(&KeyRange::lower_bound(&1u32.into(), true).unwrap())
        )
        .await,
        Ok(1)
    );
    assert_eq!(
        count_employees(
            &rexie,
            Some(&KeyRange::lower_bound(&2u32.into(), false).unwrap())
        )
        .await,
        Ok(1)
    );
    assert_eq!(
        count_employees(
            &rexie,
            Some(&KeyRange::lower_bound(&2u32.into(), true).unwrap())
        )
        .await,
        Ok(0)
    );
    assert!(clear_employees(&rexie).await.is_ok());
    assert_eq!(count_employees(&rexie, None).await, Ok(0));

    close_and_delete_db(rexie).await;
}

#[wasm_bindgen_test]
async fn test_get_all_pass() {
    let rexie = create_db().await;

    // Write values to the database.
    let id = add_employee(&rexie, "John Doe", "john@example.com").await;
    assert_eq!(id, Ok(1));

    let id2 = add_employee(&rexie, "Scooby Doo", "scooby@example.com").await;
    assert_eq!(id2, Ok(2));

    let employees = get_all_employees(&rexie).await;
    assert!(employees.is_ok());
    let employees = employees.unwrap();

    assert_eq!(employees.len(), 2);

    // TODO: check employee details

    close_and_delete_db(rexie).await;
}
