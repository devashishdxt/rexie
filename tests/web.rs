//! Test suite for the Web and headless browsers.

#![cfg(target_arch = "wasm32")]

extern crate wasm_bindgen_test;

use std::{assert, assert_eq, option::Option};

use js_sys::Array;
use rexie::{Direction, Index, KeyPath, KeyRange, ObjectStore, Result, Rexie, TransactionMode};
use serde::{Deserialize, Serialize};
use wasm_bindgen::JsValue;
use wasm_bindgen_test::*;

wasm_bindgen_test_configure!(run_in_browser);

#[derive(Debug, Deserialize, PartialEq)]
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

#[derive(Debug, Deserialize, PartialEq)]
struct Invoice {
    id: usize,
    year: u16,
    agent: String,
    customer: String,
}

#[derive(Debug, Serialize)]
struct InvoiceRequest<'a> {
    id: usize,
    year: u16,
    agent: &'a str,
    customer: &'a str,
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
        .add_object_store(
            ObjectStore::new("invoices")
                .key_path_array(["id", "year"])
                .add_index(Index::new_array("agent_customer", ["agent", "customer"])),
        )
        .build()
        .await;
    assert!(rexie.is_ok());
    rexie.unwrap()
}

/// Checks basic details of the database
async fn basic_test_db(rexie: &Rexie) {
    assert_eq!(rexie.name(), "test");
    assert_eq!(rexie.version(), Ok(1));
    assert_eq!(
        rexie.store_names(),
        vec!["departments", "employees", "invoices"]
    );

    let transaction = rexie.transaction(&["employees"], TransactionMode::ReadOnly);
    assert!(transaction.is_ok());
    let transaction = transaction.unwrap();

    assert_eq!(transaction.mode(), Ok(TransactionMode::ReadOnly));
    assert_eq!(transaction.store_names(), vec!["employees"]);

    let employees = transaction.store("employees");
    assert!(employees.is_ok());
    let employees = employees.unwrap();

    assert_eq!(employees.name(), "employees");
    assert!(employees.auto_increment());
    assert_eq!(employees.key_path(), Ok(Some(KeyPath::new_single("id"))));
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

async fn add_all_employees(rexie: &Rexie, iter: impl Iterator<Item = (&str, &str)>) -> Result<()> {
    let transaction = rexie.transaction(&["employees"], TransactionMode::ReadWrite);
    assert!(transaction.is_ok());
    let transaction = transaction.unwrap();

    let employees = transaction.store("employees");
    assert!(employees.is_ok());
    let employees = employees.unwrap();

    let requests = iter.map(|(name, email)| {
        let request = EmployeeRequest { name, email };
        let request = serde_wasm_bindgen::to_value(&request).unwrap();
        (request, None)
    });

    employees.add_all(requests).await?;

    transaction.commit().await?;
    Ok(())
}

async fn get_employee(rexie: &Rexie, id: u32) -> Result<Option<Employee>> {
    let transaction = rexie.transaction(&["employees"], TransactionMode::ReadOnly);
    assert!(transaction.is_ok());
    let transaction = transaction.unwrap();

    let employees = transaction.store("employees");
    assert!(employees.is_ok());
    let employees = employees.unwrap();

    Ok(employees
        .get(id.into())
        .await?
        .map(|value| serde_wasm_bindgen::from_value::<Employee>(value).unwrap()))
}

async fn get_all_employees(rexie: &Rexie, direction: Option<Direction>) -> Result<Vec<Employee>> {
    let transaction = rexie.transaction(&["employees"], TransactionMode::ReadOnly);
    assert!(transaction.is_ok());
    let transaction = transaction.unwrap();

    let employees = transaction.store("employees");
    assert!(employees.is_ok());
    let employees = employees.unwrap();

    let employees: Vec<JsValue> = employees
        .scan(None, None, None, direction)
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

async fn count_employees(rexie: &Rexie, key_range: Option<KeyRange>) -> Result<u32> {
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

async fn add_invoice(
    rexie: &Rexie,
    id: usize,
    year: u16,
    agent: &str,
    customer: &str,
) -> Result<()> {
    let transaction = rexie.transaction(&["invoices"], TransactionMode::ReadWrite);
    assert!(transaction.is_ok());
    let transaction = transaction.unwrap();

    let invoices = transaction.store("invoices");
    assert!(invoices.is_ok());
    let invoices = invoices.unwrap();

    let invoice = InvoiceRequest {
        id,
        year,
        agent,
        customer,
    };
    let invoice = serde_wasm_bindgen::to_value(&invoice).unwrap();
    invoices.add(&invoice, None).await?;

    transaction.commit().await?;
    Ok(())
}

async fn get_invoice(rexie: &Rexie, id: usize, year: u16) -> Result<Option<Invoice>> {
    let transaction = rexie.transaction(&["invoices"], TransactionMode::ReadOnly);
    assert!(transaction.is_ok());
    let transaction = transaction.unwrap();

    let invoices = transaction.store("invoices");
    assert!(invoices.is_ok());
    let invoices = invoices.unwrap();

    let invoice = invoices
        .get(Array::of2(&JsValue::from_f64(id as _), &JsValue::from_f64(year as _)).into())
        .await?
        .map(|value| serde_wasm_bindgen::from_value(value).unwrap());

    Ok(invoice)
}

async fn get_all_invoices_by_agent_and_customer(
    rexie: &Rexie,
    agent: &str,
    customer: &str,
) -> Result<Vec<Invoice>> {
    let transaction = rexie.transaction(&["invoices"], TransactionMode::ReadOnly);
    assert!(transaction.is_ok());
    let transaction = transaction.unwrap();

    let invoices = transaction.store("invoices");
    assert!(invoices.is_ok());
    let invoices = invoices.unwrap();

    let agent_customer_index = invoices.index("agent_customer");
    assert!(agent_customer_index.is_ok());
    let agent_customer_index = agent_customer_index.unwrap();

    let invoices = agent_customer_index
        .scan(
            Some(KeyRange::only(&Array::of2(&agent.into(), &customer.into())).unwrap()),
            None,
            None,
            None,
        )
        .await?;
    let invoices = invoices
        .into_iter()
        .map(|(_, value)| serde_wasm_bindgen::from_value(value).unwrap())
        .collect();

    Ok(invoices)
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

    let ok = add_invoice(&rexie, 1, 2022, "John Doe", "Umbrella Corp").await;
    assert!(ok.is_ok());
    let ok = add_invoice(&rexie, 1, 2023, "Scooby Doo", "Umbrella Corp").await;
    assert!(ok.is_ok());

    let invoice = get_invoice(&rexie, 1, 2022).await;
    assert!(invoice.is_ok());
    let invoice = invoice.unwrap();
    assert!(invoice.is_some());
    let invoice = invoice.unwrap();

    assert_eq!(invoice.id, 1);
    assert_eq!(invoice.year, 2022);
    assert_eq!(invoice.agent, "John Doe");
    assert_eq!(invoice.customer, "Umbrella Corp");

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
        count_employees(&rexie, Some(KeyRange::only(&1u32.into()).unwrap())).await,
        Ok(1)
    );
    assert_eq!(
        count_employees(
            &rexie,
            Some(KeyRange::lower_bound(&1u32.into(), Some(true)).unwrap())
        )
        .await,
        Ok(1)
    );
    assert_eq!(
        count_employees(
            &rexie,
            Some(KeyRange::lower_bound(&2u32.into(), Some(false)).unwrap())
        )
        .await,
        Ok(1)
    );
    assert_eq!(
        count_employees(
            &rexie,
            Some(KeyRange::lower_bound(&2u32.into(), Some(true)).unwrap())
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

    let employees = get_all_employees(&rexie, None).await;
    assert!(employees.is_ok());
    let employees = employees.unwrap();
    assert_eq!(employees.len(), 2);

    // Test reversed
    let asc_employees = get_all_employees(&rexie, Some(Direction::Next)).await;
    assert!(asc_employees.is_ok());
    let asc_employees = asc_employees.unwrap();
    let desc_employees = get_all_employees(&rexie, Some(Direction::Prev)).await;
    assert!(desc_employees.is_ok());
    let desc_employees = desc_employees.unwrap();
    assert_eq!(desc_employees[0], asc_employees[1]);
    assert_eq!(desc_employees[1], asc_employees[0]);

    // TODO: check employee details

    let ok = add_invoice(&rexie, 1, 2022, "John Doe", "Umbrella Corp").await;
    assert!(ok.is_ok());
    let ok = add_invoice(&rexie, 2, 2022, "Scooby Doo", "Umbrella Corp").await;
    assert!(ok.is_ok());
    let ok = add_invoice(&rexie, 3, 2022, "John Doe", "Umbrella Corp").await;
    assert!(ok.is_ok());

    let invoices =
        get_all_invoices_by_agent_and_customer(&rexie, "John Doe", "Umbrella Corp").await;
    assert!(invoices.is_ok());
    let invoices = invoices.unwrap();
    assert_eq!(invoices.len(), 2);
    for invoice in invoices {
        assert!(invoice.id == 1 || invoice.id == 3);
        assert_eq!(invoice.year, 2022);
    }

    close_and_delete_db(rexie).await;
}

#[wasm_bindgen_test]
async fn check_transaction_abort() {
    let rexie = create_db().await;

    let transaction = rexie.transaction(&["employees"], TransactionMode::ReadWrite);
    assert!(transaction.is_ok());
    let transaction = transaction.unwrap();

    let employees = transaction.store("employees");
    assert!(employees.is_ok());
    let employees = employees.unwrap();

    let employee = EmployeeRequest {
        name: "John Doe",
        email: "john@example.com",
    };
    let employee = serde_wasm_bindgen::to_value(&employee).unwrap();
    assert!(employees.add(&employee, None).await.is_ok());

    assert!(transaction.abort().await.is_ok());

    let employees = get_all_employees(&rexie, None).await;
    assert!(employees.is_ok());
    let employees = employees.unwrap();

    assert!(employees.is_empty());

    let id = add_employee(&rexie, "Scooby Doo", "scooby@example.com").await;
    assert_eq!(id, Ok(1));

    let employees = get_all_employees(&rexie, None).await;
    assert!(employees.is_ok());
    let employees = employees.unwrap();

    assert_eq!(employees.len(), 1);

    close_and_delete_db(rexie).await;
}

#[wasm_bindgen_test]
async fn test_add_all_pass() {
    let rexie = create_db().await;

    // Write values to the database.
    add_all_employees(
        &rexie,
        vec![
            ("John Doe", "john@example.com"),
            ("Scooby Doo", "scooby@example.com"),
        ]
        .into_iter(),
    )
    .await
    .unwrap();

    let employees = get_all_employees(&rexie, None).await;
    assert!(employees.is_ok());
    let employees = employees.unwrap();
    assert_eq!(employees.len(), 2);

    close_and_delete_db(rexie).await;
}
