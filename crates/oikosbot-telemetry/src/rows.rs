// SPDX-License-Identifier: MPL-2.0
// SPDX-FileCopyrightText: 2026 Jonathan D.A. Jewell
//! Flat row types for the estate telemetry snapshot (Parquet-backed).
//!
//! Field names/types/order are a cross-task contract: later tasks (8-13)
//! depend on these verbatim. Do not reorder or rename fields without
//! updating every downstream consumer.

/// One GitHub Actions workflow run, flattened for storage.
#[derive(
    Debug,
    Clone,
    PartialEq,
    Default,
    serde::Serialize,
    serde::Deserialize,
    parquet_derive::ParquetRecordWriter,
    parquet_derive::ParquetRecordReader,
)]
pub struct RunRow {
    pub repo: String,
    pub run_id: i64,
    pub workflow_name: String,
    pub workflow_path: String,
    pub event: String,
    pub conclusion: String,
    pub started_at: String,
    pub updated_at: String,
    pub duration_s: i64,
}

/// One repository's metadata, flattened for storage.
#[derive(
    Debug,
    Clone,
    PartialEq,
    Default,
    serde::Serialize,
    serde::Deserialize,
    parquet_derive::ParquetRecordWriter,
    parquet_derive::ParquetRecordReader,
)]
pub struct RepoRow {
    pub repo: String,
    pub visibility: String,
    pub archived: bool,
    pub pushed_at: String,
    pub size_kb: i64,
}

/// One repository release, flattened for storage.
#[derive(
    Debug,
    Clone,
    PartialEq,
    Default,
    serde::Serialize,
    serde::Deserialize,
    parquet_derive::ParquetRecordWriter,
    parquet_derive::ParquetRecordReader,
)]
pub struct ReleaseRow {
    pub repo: String,
    pub tag: String,
    pub published_at: String,
}
