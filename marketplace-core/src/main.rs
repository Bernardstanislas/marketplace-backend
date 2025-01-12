mod routes;

use dotenv::dotenv;
use log::info;
use marketplace_core::{
	application::*,
	utils::caches::{ContributorCache, RepoCache},
};
use marketplace_infrastructure::{
	database::{self, init_pool},
	github, starknet,
};

use marketplace_domain::*;
use rocket::{routes, Build, Rocket};
use rocket_okapi::{openapi_get_routes, swagger_ui::make_swagger_ui};
use slog::{o, Drain, Logger};
use std::sync::Arc;

#[macro_use]
extern crate rocket;

fn get_root_logger() -> Logger {
	let drain = match std::env::var("LOGS") {
		Ok(logs) if logs == *"terminal" => slog_async::Async::default(slog_envlogger::new(
			slog_term::CompactFormat::new(slog_term::TermDecorator::new().stderr().build())
				.build()
				.fuse(),
		)),
		_ => slog_async::Async::default(slog_envlogger::new(
			slog_json::Json::new(std::io::stdout()).add_default_keys().build().fuse(),
		)),
	};
	slog_stdlog::init().unwrap();
	slog::Logger::root(drain.fuse(), o!("version" => env!("CARGO_PKG_VERSION")))
}

#[tokio::main]
async fn main() {
	dotenv().ok();
	let root_logger = get_root_logger();
	let _global_logger_guard = slog_scope::set_global_logger(root_logger);
	github::Client::initialize();

	let database = Arc::new(database::Client::new(init_pool()));
	database.run_migrations().expect("Unable to run database migrations");

	let starknet = Arc::new(starknet::Client::default());

	let github_client = Arc::new(github::Client::new());
	let uuid_generator = Arc::new(RandomUuidGenerator);
	let contribution_repository: AggregateRootRepository<Contribution> =
		AggregateRootRepository::new(database.clone());
	let contact_information_service = Arc::new(ContactInformationServiceImplementation::new(
		database.clone(),
	));
	let application_projector: Arc<ApplicationProjector> = Arc::new(ApplicationProjector::new(
		database.clone(),
		uuid_generator.clone(),
	));

	let contribution_projector = Arc::new(ContributionProjector::new(
		database.clone(),
		github_client.clone(),
	));

	let rocket_handler = inject_app(
		rocket::build(),
		database.clone(),
		starknet,
		contribution_repository,
		contact_information_service,
		application_projector,
		contribution_projector,
		uuid_generator,
	)
	.manage(database.clone())
	.manage(RepoCache::default())
	.manage(ContributorCache::default())
	.manage(github_client)
	.attach(routes::cors::Cors)
	.mount(
		"/",
		routes![
			routes::cors::options_preflight_handler,
			routes::health::health_check,
		],
	)
	.mount(
		"/",
		openapi_get_routes![
			routes::new_project,
			routes::list_projects,
			routes::create_contribution,
			routes::assign_contributor,
			routes::validate_contribution,
			routes::unassign_contributor,
			routes::apply_to_contribution,
			routes::list_applications,
			routes::refresh_applications,
			routes::accept_application,
			routes::list_contributor_applications,
			routes::refresh_contributions,
			routes::contact_information::find_contact_information,
			routes::contact_information::put_contact_information,
		],
	)
	.mount("/swagger", make_swagger_ui(&routes::get_docs()))
	.launch();

	let (rocket_result,) = tokio::join!(rocket_handler);
	let _ = rocket_result.unwrap();

	info!("Gracefully shut down");
}

fn inject_app(
	rocket: Rocket<Build>,
	database: Arc<database::Client>,
	starknet: Arc<starknet::SingleAdminClient>,
	contribution_repository: AggregateRootRepository<Contribution>,
	contact_information_service: Arc<dyn ContactInformationService>,
	application_projector: Arc<ApplicationProjector>,
	contribution_projector: Arc<ContributionProjector>,
	uuid_generator: Arc<dyn UuidGenerator>,
) -> Rocket<Build> {
	rocket
		.manage(CreateContribution::new_usecase_boxed(starknet.clone()))
		.manage(AssignContribution::new_usecase_boxed(
			starknet.clone(),
			database.clone(),
		))
		.manage(UnassignContribution::new_usecase_boxed(
			starknet.clone(),
			database.clone(),
		))
		.manage(ApplyToContribution::new_usecase_boxed(
			contribution_repository,
			database.clone(),
			application_projector.clone(),
			uuid_generator,
		))
		.manage(ValidateContribution::new_usecase_boxed(
			starknet.clone(),
			database.clone(),
		))
		.manage(AcceptApplication::new_usecase_boxed(
			starknet,
			database.clone(),
			database.clone(),
		))
		.manage(RefreshContributions::new(
			database.clone(),
			contribution_projector,
			database.clone(),
		))
		.manage(RefreshApplications::new(
			database.clone(),
			application_projector,
			database.clone(),
		))
		.manage(database as Arc<dyn ApplicationProjectionRepository>)
		.manage(contact_information_service)
}
