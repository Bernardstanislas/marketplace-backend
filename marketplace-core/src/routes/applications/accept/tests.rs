use super::accept_application;

use marketplace_domain::{
	ApplicationId, ApplicationProjectionRepositoryError, Error as DomainError, HexPrefixedString,
};

use marketplace_core::application::AcceptApplicationUsecase;
use rocket::{
	http::{Header, Status},
	local::blocking::Client,
	Build,
};

const URI: &str = "/applications/a6127643-1344-4a44-bbfb-7142c17a4ef0/accept";

struct SuccessfulUsecase;

#[async_trait]
impl AcceptApplicationUsecase for SuccessfulUsecase {
	async fn accept_application(
		&self,
		_application_id: &ApplicationId,
	) -> Result<HexPrefixedString, DomainError> {
		Ok(HexPrefixedString::default())
	}
}

struct FailedUsecase;

#[async_trait]
impl AcceptApplicationUsecase for FailedUsecase {
	async fn accept_application(
		&self,
		_application_id: &ApplicationId,
	) -> Result<HexPrefixedString, DomainError> {
		Err(DomainError::ApplicationProjectionRepository(
			ApplicationProjectionRepositoryError::NotFound,
		))
	}
}

fn rocket(state: Box<dyn AcceptApplicationUsecase>) -> rocket::Rocket<Build> {
	rocket::build().mount("/", routes![accept_application]).manage(state)
}

#[test]
fn succes() {
	std::env::set_var("API_KEY", "ROOT");

	let client =
		Client::untracked(rocket(Box::new(SuccessfulUsecase))).expect("valid rocket instance");

	let response = client.put(URI).header(Header::new("Api-Key", "ROOT")).dispatch();

	assert_eq!(response.status(), Status::Accepted);
	assert!(response.body().is_none());
}

#[test]
fn not_found() {
	std::env::set_var("API_KEY", "ROOT");

	let client = Client::untracked(rocket(Box::new(FailedUsecase))).expect("valid rocket instance");

	let response = client.put(URI).header(Header::new("Api-Key", "ROOT")).dispatch();

	assert_eq!(response.status(), Status::NotFound);
}

#[test]
fn api_key_required() {
	std::env::set_var("API_KEY", "ROOT");

	let client = Client::untracked(rocket(Box::new(FailedUsecase))).expect("valid rocket instance");

	let response = client.put(URI).dispatch();

	assert_eq!(response.status(), Status::Unauthorized);
}

#[test]
fn api_key_must_be_valid() {
	std::env::set_var("API_KEY", "ROOT");

	let client = Client::untracked(rocket(Box::new(FailedUsecase))).expect("valid rocket instance");

	let response = client.put(URI).header(Header::new("Api-Key", "GROOT")).dispatch();

	assert_eq!(response.status(), Status::Unauthorized);
}
