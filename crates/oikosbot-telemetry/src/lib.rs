// SPDX-License-Identifier: MPL-2.0
//! Telemetry ingestion and parquet-backed storage for the estate economics pipeline.

pub mod collect;
pub mod derive;
pub mod rows;
pub mod snapshot;
