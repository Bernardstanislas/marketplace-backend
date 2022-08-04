use std::sync::Arc;

use super::list_applications;

use deathnote_contributions_feeder::domain::{
	Application, ApplicationRepository, ContributionId, ContributorId,
};
use rocket::{http::Status, local::blocking::Client, Build};
use uuid::Uuid;

const CONTRIBUTION_ID_1: &str = "a6127643-1344-4a44-bbfb-7142c17a4ef0";
struct EmptyDatabase;
impl ApplicationRepository for EmptyDatabase {
	fn store(
		&self,
		_application: deathnote_contributions_feeder::domain::Application,
	) -> Result<(), deathnote_contributions_feeder::domain::ApplicationRepositoryError> {
		unimplemented!()
	}

	fn find(
		&self,
		_id: &deathnote_contributions_feeder::domain::ApplicationId,
	) -> Result<
		Option<deathnote_contributions_feeder::domain::Application>,
		deathnote_contributions_feeder::domain::ApplicationRepositoryError,
	> {
		unimplemented!()
	}

	fn list_by_contribution(
		&self,
		_contribution_id: &ContributionId,
		_contributor_id: &Option<ContributorId>,
	) -> Result<
		Vec<deathnote_contributions_feeder::domain::Application>,
		deathnote_contributions_feeder::domain::ApplicationRepositoryError,
	> {
		Ok(vec![])
	}
}
struct FilledDatabase;
impl ApplicationRepository for FilledDatabase {
	fn store(
		&self,
		_application: deathnote_contributions_feeder::domain::Application,
	) -> Result<(), deathnote_contributions_feeder::domain::ApplicationRepositoryError> {
		unimplemented!()
	}

	fn find(
		&self,
		_id: &deathnote_contributions_feeder::domain::ApplicationId,
	) -> Result<
		Option<deathnote_contributions_feeder::domain::Application>,
		deathnote_contributions_feeder::domain::ApplicationRepositoryError,
	> {
		unimplemented!()
	}

	fn list_by_contribution(
		&self,
		_contribution_id: &ContributionId,
		_contributor_id: &Option<ContributorId>,
	) -> Result<
		Vec<deathnote_contributions_feeder::domain::Application>,
		deathnote_contributions_feeder::domain::ApplicationRepositoryError,
	> {
		Ok(vec![
			Application::new(Uuid::from_u128(0), Uuid::from_u128(0).into(), 0u128.into()),
			Application::new(Uuid::from_u128(1), Uuid::from_u128(0).into(), 1u128.into()),
		])
	}
}

fn rocket() -> rocket::Rocket<Build> {
	rocket::build().mount("/", routes![list_applications])
}

#[test]
fn ok_empty() {
	let uri = format!("/contributions/{CONTRIBUTION_ID_1}/applications");
	let client = Client::untracked(
		rocket().manage(Arc::new(EmptyDatabase) as Arc<dyn ApplicationRepository>),
	)
	.expect("valid rocket instance");
	let response = client.get(uri).dispatch();

	assert_eq!(response.status(), Status::Ok);

	assert_eq!(
		Vec::<Application>::new(),
		response.into_json::<Vec<Application>>().unwrap()
	);
}

#[test]
fn ok_multiple() {
	let uri = format!("/contributions/{CONTRIBUTION_ID_1}/applications");
	let client = Client::untracked(
		rocket().manage(Arc::new(FilledDatabase) as Arc<dyn ApplicationRepository>),
	)
	.expect("valid rocket instance");
	let response = client.get(uri).dispatch();

	assert_eq!(response.status(), Status::Ok);
	assert_eq!(
		vec![
			Application::new(Uuid::from_u128(0), Uuid::from_u128(0).into(), 0u128.into()),
			Application::new(Uuid::from_u128(1), Uuid::from_u128(0).into(), 1u128.into()),
		],
		response.into_json::<Vec<Application>>().unwrap()
	);
}

#[test]
fn ok_specifying_contributor() {
	let uri = format!(
		"/contributions/{CONTRIBUTION_ID_1}/applications?contributor_id=0x0000000000000000000000000000000000000000000000000000000000000000"
	);
	let client = Client::untracked(
		rocket().manage(Arc::new(EmptyDatabase) as Arc<dyn ApplicationRepository>),
	)
	.expect("valid rocket instance");
	let response = client.get(uri).dispatch();

	assert_eq!(response.status(), Status::Ok);
	assert_eq!(
		Vec::<Application>::new(),
		response.into_json::<Vec<Application>>().unwrap()
	);
}