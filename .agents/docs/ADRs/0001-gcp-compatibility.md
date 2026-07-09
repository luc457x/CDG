# GCP & Vertex AI Compatibility

**Status:** accepted  
**Date:** 2026-06-13

### Context & Problem Statement
The project previously contemplated a full migration of caching database and storage layers to Google Cloud Platform (GCP) and establishing a native Vertex AI training/inference pipeline. However, full cloud migration introduces unnecessary cost, hosting complexity, and reduces ease of local development. 

Instead of doing a complete migration to GCP, we need to ensure that the project is designed with full compatibility with GCP (Cloud SQL, BigQuery, GCS) and Vertex AI downstream workflows, allowing seamless deployment or consumption inside GCP environments when needed.

### Decision
Decided to maintain **architectural and format compatibility with GCP and Vertex AI** instead of performing a full cloud migration:
1. **Database Queries**: Keep SQLite DB layer and compile-time verified queries (`sqlx`) structured to easily port/migrate to GCP Cloud SQL (PostgreSQL/MySQL) if required.
2. **Output Standardization**: Export all cleaned/pre-processed datasets strictly in Parquet and CSV formats to be directly compatible with Google Cloud Storage ingestion, BigQuery federated queries, and Vertex AI AutoML/custom container pipelines.
3. **Resource Footprint**: Keep application memory and CPU footprint within the boundaries of GCP Free Tier (Cloud Run / e2-micro instances) to support zero-cost cron/job executions (when using `light` mode).

### Consequences
- **Good:** Lower operating costs, zero vendor lock-in, and simpler local development/testing.
- **Good:** Easy ingestion to BigQuery/Vertex AI from GCS buckets.
- **Bad:** Syncing cache databases across distributed runner instances requires file sharing or database replication if not using central Cloud SQL.
