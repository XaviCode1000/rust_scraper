//! Dependency Injection Container
//!
//! Provides a centralized way to wire up all services and their dependencies.
//! Following Clean Architecture, the container lives in the application layer
//! and creates instances of infrastructure implementations.

use std::sync::Arc;

use crate::application::crawl_options::CrawlOptions;
use crate::application::crawl_result_repository::CrawlResultRepositoryImpl;
use crate::application::elastic_ingestion::ElasticIngestion;
use crate::application::http_client::{HttpClient, HttpClientConfig};
use crate::domain::repository::DynVectorRepository;
use crate::domain::{repositories::CrawlResultRepository, CrawlerConfig};
use crate::infrastructure::autotuning::ElasticConfig;
use crate::infrastructure::bridge::CpuBridge;
use crate::infrastructure::config::ScraperConfig;
use crate::infrastructure::cpu_pool::RayonCpuPool;
use crate::infrastructure::crawler::resource_downloader::{DownloadConfig, ResourceDownloader};
use crate::infrastructure::export::state_store::StateStore;
// SQLite persistence layer — only compiled under the `persistence` feature.
#[cfg(feature = "persistence")]
use crate::infrastructure::persistence::sqlite::{
    self as sqlite_persistence, SqliteVectorRepository,
};

/// Dependency Injection Container
///
/// Holds all service instances and their configurations.
/// Services are created once and reused throughout the application.
#[derive(Clone)]
pub struct Container {
    pub crawler_config: CrawlerConfig,
    pub scraper_config: ScraperConfig,
    pub http_client: Arc<HttpClient>,
    pub state_store: Option<Arc<StateStore>>,
    pub crawl_result_repo: Option<Arc<dyn CrawlResultRepository>>,
    /// Elastic ingestion pipeline (optional, activated via `--elastic` or
    /// `--output-vectors`). Erased to `DynVectorRepository` so it can hold either
    /// the SQLite repo (`persistence` feature) or the headless `StreamRepository`
    /// JSONL sink.
    pub elastic_ingestion: Option<Arc<ElasticIngestion<DynVectorRepository>>>,
}

impl Container {
    /// Create a new container with the given configurations.
    ///
    /// # Arguments
    ///
    /// * `crawler_config` - Configuration for crawling behavior
    /// * `scraper_config` - Configuration for scraping behavior
    ///
    /// # Returns
    ///
    /// A new container instance with all services initialized
    pub async fn new(
        crawler_config: CrawlerConfig,
        scraper_config: ScraperConfig,
    ) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        // Initialize infrastructure services
        let http_client = Arc::new(HttpClient::new(HttpClientConfig::default())?);

        // State store is optional (for resume mode)
        let state_store = None;

        // Crawl result repository using append-only log
        let log_path = scraper_config.output_dir.join("crawl_results.bin");
        let crawl_result_repo = match CrawlResultRepositoryImpl::new(log_path, 1024) {
            Ok(repo) => Some(Arc::new(repo) as Arc<dyn CrawlResultRepository>),
            Err(e) => {
                tracing::warn!("no se pudo inicializar el repositorio: {e}");
                None
            },
        };

        Ok(Self {
            crawler_config,
            scraper_config,
            http_client,
            state_store,
            crawl_result_repo,
            elastic_ingestion: None,
        })
    }

    /// Set the state store for resume functionality.
    pub fn with_state_store(mut self, state_store: StateStore) -> Self {
        self.state_store = Some(Arc::new(state_store));
        self
    }

    /// Get a repository for crawl results (backed by append-only log).
    pub fn crawl_result_repository(&self) -> Option<Arc<dyn CrawlResultRepository>> {
        self.crawl_result_repo.clone()
    }

    /// Build the elastic ingestion pipeline around an arbitrary repository.
    ///
    /// Shared by the SQLite path (`persistence` feature) and the headless
    /// `StreamRepository` JSONL sink. Wires `RayonCpuPool` → `CpuBridge` →
    /// `ResourceDownloader` (byte-weighted semaphore) → `ElasticIngestion`.
    ///
    /// # Errors
    ///
    /// Returns an error if the Rayon pool or HTTP client fails to initialize.
    fn build_elastic(
        repository: DynVectorRepository,
        config: &ElasticConfig,
    ) -> Result<ElasticIngestion<DynVectorRepository>, Box<dyn std::error::Error + Send + Sync>>
    {
        // 1. Rayon CPU pool for lol_html processing
        let cpu_pool = RayonCpuPool::new(config.cpu_cores)?;

        // 2. CpuBridge wraps the Rayon pool with catch_unwind safety
        let bridge = CpuBridge::new(cpu_pool);

        // 3. HTTP client for resource downloads (separate from scraping client)
        let client = crate::application::http_client::create_http_client()?;
        let max_concurrent = (config.ram_budget_bytes / config.max_resource_bytes).max(1) as usize;
        let semaphore = Arc::new(tokio::sync::Semaphore::new(max_concurrent));

        // 4. Resource downloader with elastic semaphore (byte-weighted backpressure)
        let downloader = ResourceDownloader::with_config(
            semaphore,
            client,
            DownloadConfig {
                max_size_bytes: config.max_resource_bytes,
                ..Default::default()
            },
        );

        // 5. Assemble pipeline — ElasticIngestion erased to DynVectorRepository
        let autotune = crate::infrastructure::config::AutotuningConfig::from_elastic(config);
        Ok(ElasticIngestion::new(
            downloader, bridge, repository, autotune,
        ))
    }

    /// Activate the elastic ingestion pipeline with SQLite persistence.
    ///
    /// Resolves `ElasticConfig` from the provided options, then wires
    /// `RayonCpuPool` → `CpuBridge` → `SqliteVectorRepository` →
    /// `ResourceDownloader` → `ElasticIngestion`. Only available under the
    /// `persistence` feature.
    ///
    /// # Errors
    ///
    /// Returns an error if the Rayon pool or SQLite pool fails to initialize.
    #[cfg(feature = "persistence")]
    pub async fn with_elastic(
        mut self,
        opts: &CrawlOptions,
    ) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let overrides = crate::infrastructure::autotuning::ElasticOverrides {
            cpu_cores: opts.elastic.cpu_cores,
            ram_budget_bytes: opts.elastic.ram_budget_bytes,
            max_resource_bytes: opts.elastic.max_resource_bytes,
            db_path: opts.elastic.db_path.clone(),
        };
        let config = ElasticConfig::resolve(&overrides);

        // 3. SQLite pool → repository (WAL mode, auto-creates parent dir)
        let pool = sqlite_persistence::create_pool(&config.db_path, config.db_pool_size)?;
        sqlite_persistence::setup_schema(&pool).await?;
        let repository: DynVectorRepository = Arc::new(SqliteVectorRepository::new(pool));

        let ingestion = Self::build_elastic(repository, &config)?;
        self.elastic_ingestion = Some(Arc::new(ingestion));
        Ok(self)
    }

    /// Access the elastic ingestion pipeline, if activated.
    #[must_use]
    pub fn elastic_ingestion(&self) -> Option<&ElasticIngestion<DynVectorRepository>> {
        self.elastic_ingestion.as_deref()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::CrawlerConfig;
    use crate::infrastructure::config::ScraperConfig;
    use tempfile::TempDir;

    #[cfg_attr(miri, ignore = "boring-sys2 FFI (wreq Client) not supported by Miri")]
    #[tokio::test]
    async fn test_container_wires_crawl_result_repository() {
        let tmp = TempDir::new().unwrap();
        let crawler_config = CrawlerConfig::new(url::Url::parse("https://example.com").unwrap());
        let scraper_config = ScraperConfig {
            output_dir: tmp.path().to_path_buf(),
            ..Default::default()
        };

        let container = Container::new(crawler_config, scraper_config)
            .await
            .unwrap();
        let repo = container.crawl_result_repository();
        assert!(
            repo.is_some(),
            "crawl_result_repository() debe retornar Some"
        );
    }
}
