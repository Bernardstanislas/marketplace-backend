use crate::database::{init_pool, Client};
use assert_matches::assert_matches;
use marketplace_domain::*;
use uuid::Uuid;

use super::init_contribution;

#[test]
#[cfg_attr(
	not(feature = "with_infrastructure_tests"),
	ignore = "infrastructure test"
)]
fn store_and_find() {
	let client = Client::new(init_pool());

	let contribution = init_contribution(&client);

	let application1 =
		ApplicationProjection::new(Uuid::new_v4().into(), contribution.id.clone(), 0.into());
	let application2 = ApplicationProjection::new(Uuid::new_v4().into(), contribution.id, 1.into());

	<Client as ApplicationProjectionRepository>::create(&client, application1.clone()).unwrap();
	<Client as ApplicationProjectionRepository>::create(&client, application2.clone()).unwrap();

	let found_application =
		<Client as ApplicationProjectionRepository>::find(&client, application1.id()).unwrap();
	assert_eq!(found_application, Some(application1));

	let found_application =
		<Client as ApplicationProjectionRepository>::find(&client, application2.id()).unwrap();
	assert_eq!(found_application, Some(application2));

	let found_application =
		<Client as ApplicationProjectionRepository>::find(&client, &Uuid::new_v4().into()).unwrap();
	assert_eq!(found_application, None);
}

#[test]
#[cfg_attr(
	not(feature = "with_infrastructure_tests"),
	ignore = "infrastructure test"
)]
fn id_must_be_unique() {
	let client = Client::new(init_pool());

	let contribution = init_contribution(&client);

	let id = Uuid::new_v4().into();

	let application1 = ApplicationProjection::new(id, contribution.id.clone(), 0.into());
	let application2 = ApplicationProjection::new(id, contribution.id, 1.into());

	<Client as ApplicationProjectionRepository>::create(&client, application1).unwrap();
	let res = <Client as ApplicationProjectionRepository>::create(&client, application2);

	assert!(res.is_err());
	assert_matches!(
		res.unwrap_err(),
		ApplicationProjectionRepositoryError::AlreadyExist(_)
	);
}

#[test]
#[cfg_attr(
	not(feature = "with_infrastructure_tests"),
	ignore = "infrastructure test"
)]
fn find_return_none_if_not_found() {
	let client = Client::new(init_pool());

	let res = <Client as ApplicationProjectionRepository>::find(&client, &Uuid::new_v4().into());
	assert!(res.is_ok());
	assert_eq!(res.unwrap(), None)
}

#[test]
#[cfg_attr(
	not(feature = "with_infrastructure_tests"),
	ignore = "infrastructure test"
)]
fn cannot_apply_twice() {
	let client = Client::new(init_pool());

	let contribution = init_contribution(&client);

	let application = ApplicationProjection::new(Uuid::new_v4().into(), contribution.id, 0.into());

	<Client as ApplicationProjectionRepository>::create(&client, application.clone()).unwrap();
	let res = <Client as ApplicationProjectionRepository>::create(&client, application);

	assert!(res.is_err());
	assert_matches!(
		res.unwrap_err(),
		ApplicationProjectionRepositoryError::AlreadyExist(_)
	);
}

#[test]
#[cfg_attr(
	not(feature = "with_infrastructure_tests"),
	ignore = "infrastructure test"
)]
fn store_multiple_and_list() {
	let client = Client::new(init_pool());

	let contribution = init_contribution(&client);

	let application1 =
		ApplicationProjection::new(Uuid::new_v4().into(), contribution.id.clone(), 0.into());

	let application2 =
		ApplicationProjection::new(Uuid::new_v4().into(), contribution.id.clone(), 1.into());

	<Client as ApplicationProjectionRepository>::create(&client, application1.clone()).unwrap();
	<Client as ApplicationProjectionRepository>::create(&client, application2.clone()).unwrap();

	let applications = <Client as ApplicationProjectionRepository>::list_by_contribution(
		&client,
		&contribution.id,
		None,
	)
	.unwrap();

	assert_eq!(
		applications,
		vec![application1.clone(), application2.clone()]
	);

	let applications = <Client as ApplicationProjectionRepository>::list_by_contribution(
		&client,
		&contribution.id,
		Some(application1.contributor_id().to_owned()),
	)
	.unwrap();

	assert_eq!(applications, vec![application1]);

	let applications = <Client as ApplicationProjectionRepository>::list_by_contribution(
		&client,
		&contribution.id,
		Some(application2.contributor_id().to_owned()),
	)
	.unwrap();

	assert_eq!(applications, vec![application2])
}
