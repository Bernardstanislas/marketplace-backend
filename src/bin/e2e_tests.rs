use reqwest::StatusCode;
use std::{fs::read_to_string, str::FromStr, thread, time::Duration};

mod projects;
use projects::*;

mod utils;
use utils::*;

mod contribution;
use contribution::*;

#[tokio::main]
async fn main() {
	add_all_projects().await;

	let starkonquest = find_project_by_title(&list_all_projects().await, "starkonquest");
	let starkonquest_id: u64 = starkonquest["id"]
		.as_str()
		.expect("starkonquest id is not a string")
		.parse()
		.expect("starkonquest id is not a number");

	const ISSUE_NUMBER: u64 = 51;
	add_contribution(ISSUE_NUMBER, starkonquest_id, 0, "0x123").await;

	wait_for_result("src/data/contributions_open.json").await;

	let starkonquest = find_project_by_title(&list_all_projects().await, "starkonquest");
	let contribution =
		find_contribution_by_onchain_id(&starkonquest, starkonquest_id * 1_000_000 + ISSUE_NUMBER);

	let contribution_id =
		uuid::Uuid::from_str(contribution["id"].as_str().expect("contribution id is not a str"))
			.expect("contribution id is not a valid uuid");

	const CONTRIBUTOR_ID: u128 = 123;
	assign_contribution(contribution_id, CONTRIBUTOR_ID).await;

	wait_for_result("src/data/contributions_assigned.json").await;

	validate_contribution(contribution_id).await;

	wait_for_result("src/data/contributions_validated.json").await;
}

async fn add_all_projects() {
	add_project("onlydustxyz", "starkonquest").await;
	add_project("onlydustxyz", "starklings").await;
}

async fn wait_for_result(result_file_path: &'static str) {
	for _ in 0..10 {
		let handle = tokio::spawn(async move {
			compare_jsons(
				list_all_projects().await,
				serde_json::from_str(&read_to_string(result_file_path).unwrap()).unwrap(),
			);
		});

		match tokio::join!(handle).0 {
			Ok(_) => return,
			Err(_) => {
				thread::sleep(Duration::from_secs(3));
			},
		}
	}

	assert!(false, "Result does not match with {result_file_path}");
}