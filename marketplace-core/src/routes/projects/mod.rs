use marketplace_core::{dto, utils::caches};
use marketplace_domain::*;
use marketplace_infrastructure::{database, github, starknet};

use futures::future::{self, OptionFuture};
use http_api_problem::{HttpApiProblem, StatusCode};
use log::{error, warn};
use rocket::{get, http::Status, post, serde::json::Json, State};
use rocket_okapi::openapi;
use std::{error::Error, result::Result, sync::Arc};
use url::Url;

use super::api_key::ApiKey;

#[openapi(tag = "Projects")]
#[post("/projects", format = "application/json", data = "<project>")]
pub async fn new_project(
	_api_key: ApiKey,
	project: Json<dto::ProjectCreation<'_>>,
	database: &State<Arc<database::Client>>,
	github: &State<Arc<github::Client>>,
) -> Result<Status, HttpApiProblem> {
	let project = github
		.get_project_by_owner_and_name(project.owner, project.name)
		.await
		.map_err(|error| {
			let mut problem = HttpApiProblem::new(StatusCode::INTERNAL_SERVER_ERROR)
				.title("Fetching projects failed");
			if let Some(s) = error.source() {
				problem.detail = Some(s.to_string());
			}
			problem
		})?;

	ProjectRepository::store(database.as_ref(), project).map_err(|error| {
		let mut problem = HttpApiProblem::new(StatusCode::INTERNAL_SERVER_ERROR)
			.title("Saving projects to DB failed");
		if let Some(s) = error.source() {
			problem.detail = Some(s.to_string());
		}
		problem
	})?;

	Ok(Status::Accepted)
}

#[openapi(tag = "Projects")]
#[get("/projects")]
pub async fn list_projects(
	database: &State<Arc<database::Client>>,
	repo_cache: &State<caches::RepoCache>,
	contributor_cache: &State<caches::ContributorCache>,
) -> Result<Json<Vec<dto::Project>>, HttpApiProblem> {
	let projects_with_contribution_iterator = database
		.find_all_with_contributions()
		.map_err(|error| {
			let mut problem = HttpApiProblem::new(StatusCode::INTERNAL_SERVER_ERROR)
				.title("Listing projects failed");
			if let Some(s) = error.source() {
				problem.detail = Some(s.to_string());
			}
			problem
		})?
		.into_iter();

	// Spawn concurent tasks
	// One for each project
	let build_project_tasks = projects_with_contribution_iterator.map(|project| {
		let cloned_repo_cache: caches::RepoCache = repo_cache.inner().clone();
		let cloned_contributor_cache: caches::ContributorCache = contributor_cache.inner().clone();
		tokio::spawn(async move {
			build_project(project, &cloned_repo_cache, &cloned_contributor_cache).await
		})
	});

	// Merge all tasks into a single vector
	// Failed task will be ignored
	let projects = future::join_all(build_project_tasks)
		.await
		.into_iter()
		.filter_map(|result| match result {
			Ok(opt_project) => opt_project,
			Err(e) => {
				error!("failed to build project dto: {}", e.to_string());
				None
			},
		})
		.collect();

	Ok(Json(projects))
}

async fn build_project(
	project: ProjectWithContributions,
	repo_cache: &caches::RepoCache,
	contributor_cache: &caches::ContributorCache,
) -> Option<dto::Project> {
	let github_repository = repo_cache
		.inner_ref()
		.get_or_insert(&project.project.id, || async {
			match github::Client::new().repository_by_id(project.project.id).await {
				Ok(repo) => Some(repo),
				Err(e) => {
					warn!("Unable to fetch repository from GitHub: {}", e.to_string());
					None
				},
			}
		})
		.await?;

	// Spawn concurent tasks
	// One for each contribution
	let build_contribution_tasks = project.contributions.into_iter().map(|contribution| {
		let cloned_contributor_cache = contributor_cache.clone();
		tokio::spawn(
			async move { build_contribution(contribution, &cloned_contributor_cache).await },
		)
	});

	// Merge all tasks into a single vector
	// Failed task will be ignored
	let contributions = future::join_all(build_contribution_tasks)
		.await
		.into_iter()
		.filter_map(|result| match result {
			Ok(opt_project) => opt_project,
			Err(e) => {
				error!("failed to build contribution dto: {}", e.to_string());
				None
			},
		})
		.collect();

	let project = dto::Project {
		id: project.project.id.to_string(),
		title: project.project.name.clone(),
		description: github_repository.description,
		logo: github_repository.owner.unwrap().avatar_url,
		github_link: github_repository.html_url.unwrap_or_else(|| {
			Url::parse(&format!(
				"https://github.com/{}/{}",
				project.project.owner, project.project.name
			))
			.unwrap()
		}),
		contributions,
	};

	Some(project)
}

async fn build_contribution(
	contribution: ContributionProjection,
	contributor_cache: &caches::ContributorCache,
) -> Option<dto::Contribution> {
	let contributor = OptionFuture::from(
		contribution
			.contributor_id
			.clone()
			.map(|id| build_contributor(contributor_cache, id)),
	)
	.await
	.flatten();

	let mut contribution = dto::Contribution::from(contribution);

	if contributor.is_some() {
		contribution.metadata.github_username = contributor.unwrap().github_username;
	}

	Some(contribution)
}

async fn build_contributor(
	contributor_cache: &caches::ContributorCache,
	contributor_id: ContributorId,
) -> Option<Contributor> {
	contributor_cache
		.inner_ref()
		.get_or_insert(&contributor_id, || async {
			fetch_contributor(&contributor_id).await
		})
		.await
}

async fn fetch_contributor(contributor_id: &ContributorId) -> Option<Contributor> {
	let starknet = starknet::Client::default();
	let mut contributor = starknet.get_user_information(contributor_id).await?;

	if let Some(github_handle) = &contributor.github_handle {
		let github_user = match github::Client::new().user(github_handle).await {
			Ok(user) => Some(user),
			Err(e) => {
				warn!("Unable to fetch user from GitHub: {}", e.to_string());
				None
			},
		};

		contributor.github_username = github_user.map(|u| u.login);
	}

	Some(contributor)
}
