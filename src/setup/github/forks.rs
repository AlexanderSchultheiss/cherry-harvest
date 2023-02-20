//! Using GitHub's forks API.

use octocrab::models::Repository;
use octocrab::Octocrab;
use reqwest::Url;

#[async_trait::async_trait]
pub trait ForksExt {
    fn forks(&self) -> ForksHandler;
}

#[async_trait::async_trait]
impl ForksExt for Octocrab {
    fn forks(&self) -> ForksHandler {
        ForksHandler::new(self)
    }
}

/// Handler for the forks API.
pub struct ForksHandler<'octo> {
    crab: &'octo Octocrab,
}

impl<'octo> ForksHandler<'octo> {
    pub(crate) fn new(crab: &'octo Octocrab) -> Self {
        Self { crab }
    }

    /// Lists all forks of the repository.
    /// ```
    pub fn list<'q>(self, forks_url: Url) -> QueryHandler<'octo, 'q, Repository> {
        QueryHandler::new(self.crab, forks_url, "")
    }
}

#[derive(Clone, Debug)]
pub enum ContentType {
    TextMatch,
    Default,
}

impl Default for ContentType {
    fn default() -> Self {
        Self::Default
    }
}

/// A handler for handling search queries to GitHub.
#[derive(Clone, Debug, serde::Serialize)]
pub struct QueryHandler<'octo, 'query, T> {
    #[serde(skip)]
    return_type: std::marker::PhantomData<T>,
    #[serde(skip)]
    crab: &'octo Octocrab,
    #[serde(skip)]
    url: Url,
    #[serde(skip)]
    content_type: ContentType,
    #[serde(rename = "q")]
    query: &'query str,
    per_page: Option<u8>,
    page: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    sort: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    order: Option<String>,
}

impl<'octo, 'query, T> QueryHandler<'octo, 'query, T> {
    pub(crate) fn new(crab: &'octo Octocrab, url: Url, query: &'query str) -> Self {
        Self {
            content_type: ContentType::Default,
            crab,
            order: None,
            page: None,
            per_page: None,
            query,
            return_type: std::marker::PhantomData,
            url,
            sort: None,
        }
    }

    /// Sets the `sort` parameter for the query. The exact parameters for this
    /// method will vary based on what is being searched.
    pub fn sort<S: Into<String>>(mut self, sort: impl Into<Option<S>>) -> Self {
        self.sort = sort.into().map(S::into);
        self
    }

    /// Results per page (max 100).
    pub fn per_page(mut self, per_page: impl Into<u8>) -> Self {
        self.per_page = Some(per_page.into());
        self
    }

    /// Page number of the results to fetch.
    pub fn page(mut self, page: impl Into<u32>) -> Self {
        self.page = Some(page.into());
        self
    }
}

impl<'octo, 'query, T: serde::de::DeserializeOwned> QueryHandler<'octo, 'query, T> {
    /// Send the actual request.
    pub async fn send(self) -> octocrab::Result<octocrab::Page<T>> {
        self.crab.get(&format!("{}", self.url), Some(&self)).await
    }
}
