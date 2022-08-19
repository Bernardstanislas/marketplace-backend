use deathnote_contributions_feeder::domain::{
	ApplicationRepositoryError, ApplicationServiceError, ContactInformationRepositoryError,
	ContributionRepositoryError, ContributionServiceError, DomainError,
	OnchainContributionServiceError,
};
use http_api_problem::{HttpApiProblem, StatusCode};

pub(crate) trait ToHttpApiProblem {
	fn to_http_api_problem(&self) -> HttpApiProblem;
}

impl ToHttpApiProblem for ApplicationRepositoryError {
	fn to_http_api_problem(&self) -> HttpApiProblem {
		match self {
			ApplicationRepositoryError::NotFound =>
				HttpApiProblem::new(StatusCode::NOT_FOUND).title(self.to_string()),
			ApplicationRepositoryError::AlreadyExist(e) =>
				HttpApiProblem::new(StatusCode::CONFLICT)
					.title(self.to_string())
					.detail(e.to_string()),
			ApplicationRepositoryError::InvalidEntity(e) =>
				HttpApiProblem::new(StatusCode::BAD_REQUEST)
					.title(self.to_string())
					.detail(e.to_string()),
			ApplicationRepositoryError::Infrastructure(e) =>
				HttpApiProblem::new(StatusCode::INTERNAL_SERVER_ERROR)
					.title(self.to_string())
					.detail(e.to_string()),
		}
	}
}

impl ToHttpApiProblem for ContributionRepositoryError {
	fn to_http_api_problem(&self) -> HttpApiProblem {
		match self {
			ContributionRepositoryError::NotFound =>
				HttpApiProblem::new(StatusCode::NOT_FOUND).title(self.to_string()),
			ContributionRepositoryError::AlreadyExist(e) =>
				HttpApiProblem::new(StatusCode::BAD_REQUEST)
					.title(self.to_string())
					.detail(e.to_string()),
			ContributionRepositoryError::InvalidEntity(e) =>
				HttpApiProblem::new(StatusCode::BAD_REQUEST)
					.title(self.to_string())
					.detail(e.to_string()),
			ContributionRepositoryError::Infrastructure(e) =>
				HttpApiProblem::new(StatusCode::INTERNAL_SERVER_ERROR)
					.title(self.to_string())
					.detail(e.to_string()),
		}
	}
}

impl ToHttpApiProblem for ContactInformationRepositoryError {
	fn to_http_api_problem(&self) -> HttpApiProblem {
		match self {
			ContactInformationRepositoryError::NotFound =>
				HttpApiProblem::new(StatusCode::NOT_FOUND).title(self.to_string()),
			ContactInformationRepositoryError::Infrastructure(e) =>
				HttpApiProblem::new(StatusCode::INTERNAL_SERVER_ERROR)
					.title(self.to_string())
					.detail(e.to_string()),
		}
	}
}

impl ToHttpApiProblem for OnchainContributionServiceError {
	fn to_http_api_problem(&self) -> HttpApiProblem {
		match self {
			OnchainContributionServiceError::Infrastructure(e) =>
				HttpApiProblem::new(StatusCode::INTERNAL_SERVER_ERROR)
					.title(self.to_string())
					.detail(e.to_string()),
		}
	}
}

impl ToHttpApiProblem for ContributionServiceError {
	fn to_http_api_problem(&self) -> HttpApiProblem {
		match self {
			ContributionServiceError::CannotApply(e) =>
				HttpApiProblem::new(StatusCode::BAD_REQUEST)
					.title(self.to_string())
					.detail(e.to_string()),
		}
	}
}

impl ToHttpApiProblem for ApplicationServiceError {
	fn to_http_api_problem(&self) -> HttpApiProblem {
		match self {
			ApplicationServiceError::NotFound =>
				HttpApiProblem::new(StatusCode::NOT_FOUND).title(self.to_string()),
			ApplicationServiceError::AlreadyExist(e) =>
				HttpApiProblem::new(StatusCode::BAD_REQUEST)
					.title(self.to_string())
					.detail(e.to_string()),
			ApplicationServiceError::InvalidEntity(e) =>
				HttpApiProblem::new(StatusCode::BAD_REQUEST)
					.title(self.to_string())
					.detail(e.to_string()),
			ApplicationServiceError::Infrastructure(e) =>
				HttpApiProblem::new(StatusCode::INTERNAL_SERVER_ERROR)
					.title(self.to_string())
					.detail(e.to_string()),
			ApplicationServiceError::InvalidApplicationStatus { .. } =>
				HttpApiProblem::new(StatusCode::BAD_REQUEST).title(self.to_string()),
			ApplicationServiceError::InvalidContributionStatus { .. } =>
				HttpApiProblem::new(StatusCode::BAD_REQUEST).title(self.to_string()),
		}
	}
}

impl ToHttpApiProblem for DomainError {
	fn to_http_api_problem(&self) -> HttpApiProblem {
		match self {
			DomainError::ApplicationRepository(application_repository_error) =>
				application_repository_error.to_http_api_problem(),
			DomainError::ContributionRepository(contribution_repository_error) =>
				contribution_repository_error.to_http_api_problem(),
			DomainError::ContactInformationRepository(contact_information_repository_error) =>
				contact_information_repository_error.to_http_api_problem(),
			DomainError::OnchainContributionService(onchain_contribution_service_error) =>
				onchain_contribution_service_error.to_http_api_problem(),
			DomainError::ContributionService(contribution_service_error) =>
				contribution_service_error.to_http_api_problem(),
			DomainError::ApplicationService(application_service_error) =>
				application_service_error.to_http_api_problem(),
			DomainError::Lock =>
				HttpApiProblem::new(StatusCode::INTERNAL_SERVER_ERROR).title(self.to_string()),
		}
	}
}