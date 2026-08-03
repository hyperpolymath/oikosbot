// SPDX-License-Identifier: MPL-2.0
// SPDX-FileCopyrightText: 2026 Jonathan D.A. Jewell
//! Parquet-backed snapshot read/write for telemetry row types.
//!
//! Adapted for `parquet` 59.1.0: writing uses `parquet_derive::ParquetRecordWriter`
//! (`RecordWriter` trait impl on `&[T]`); reading uses `parquet_derive::ParquetRecordReader`
//! (`RecordReader` trait impl on `Vec<T>`), which is simpler and more robust than the
//! `RowAccessor` column-index approach in the original brief (that reference targeted an
//! older parquet API).

use crate::rows::{ReleaseRow, RepoRow, RunRow};
use anyhow::{Context, Result};
use parquet::file::properties::WriterProperties;
use parquet::file::reader::{FileReader, SerializedFileReader};
use parquet::file::writer::SerializedFileWriter;
use parquet::record::{RecordReader, RecordWriter};
use std::fs::File;
use std::path::Path;
use std::sync::Arc;

/// Write `rows` to `path` as a single-row-group Parquet file.
fn write_rows<T>(path: &Path, rows: &[T]) -> Result<()>
where
    for<'a> &'a [T]: RecordWriter<T>,
{
    let schema = rows
        .schema()
        .with_context(|| format!("derive parquet schema for {}", path.display()))?;
    let file =
        File::create(path).with_context(|| format!("create parquet file {}", path.display()))?;
    let props = Arc::new(WriterProperties::builder().build());
    let mut writer = SerializedFileWriter::new(file, schema, props)
        .with_context(|| format!("open parquet writer for {}", path.display()))?;
    let mut row_group = writer.next_row_group()?;
    rows.write_to_row_group(&mut row_group)?;
    row_group.close()?;
    writer.close()?;
    Ok(())
}

/// Read all rows back from a Parquet file written by [`write_rows`].
fn read_rows<T>(path: &Path) -> Result<Vec<T>>
where
    Vec<T>: RecordReader<T>,
{
    let file = File::open(path).with_context(|| format!("open parquet file {}", path.display()))?;
    let reader = SerializedFileReader::new(file)
        .with_context(|| format!("open parquet reader for {}", path.display()))?;
    let mut out: Vec<T> = Vec::new();
    for (i, rg_meta) in reader.metadata().row_groups().iter().enumerate() {
        let mut row_group = reader.get_row_group(i)?;
        out.read_from_row_group(&mut *row_group, rg_meta.num_rows() as usize)?;
    }
    Ok(out)
}

/// Write [`RunRow`]s to a Parquet snapshot at `path`.
pub fn write_runs(path: &Path, rows: &[RunRow]) -> Result<()> {
    write_rows(path, rows)
}

/// Read [`RunRow`]s back from a Parquet snapshot at `path`.
pub fn read_runs(path: &Path) -> Result<Vec<RunRow>> {
    read_rows(path)
}

/// Write [`RepoRow`]s to a Parquet snapshot at `path`.
pub fn write_repos(path: &Path, rows: &[RepoRow]) -> Result<()> {
    write_rows(path, rows)
}

/// Read [`RepoRow`]s back from a Parquet snapshot at `path`.
pub fn read_repos(path: &Path) -> Result<Vec<RepoRow>> {
    read_rows(path)
}

/// Write [`ReleaseRow`]s to a Parquet snapshot at `path`.
pub fn write_releases(path: &Path, rows: &[ReleaseRow]) -> Result<()> {
    write_rows(path, rows)
}

/// Read [`ReleaseRow`]s back from a Parquet snapshot at `path`.
pub fn read_releases(path: &Path) -> Result<Vec<ReleaseRow>> {
    read_rows(path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rows::RunRow;

    fn temp_path(name: &str) -> std::path::PathBuf {
        let dir = std::env::temp_dir().join("oikos-snap-test");
        std::fs::create_dir_all(&dir).unwrap();
        dir.join(name)
    }

    #[test]
    fn parquet_roundtrip_preserves_rows() {
        let rows = vec![RunRow {
            repo: "o/r".into(),
            run_id: 1,
            workflow_name: "CI".into(),
            workflow_path: ".github/workflows/ci.yml".into(),
            event: "push".into(),
            conclusion: "success".into(),
            started_at: "2026-08-01T00:00:00Z".into(),
            updated_at: "2026-08-01T00:05:00Z".into(),
            duration_s: 300,
        }];
        let p = temp_path("runs.parquet");
        write_runs(&p, &rows).unwrap();
        let back = read_runs(&p).unwrap();
        assert_eq!(back.len(), 1);
        assert_eq!(back[0].repo, "o/r");
        assert_eq!(back[0].duration_s, 300);
        assert_eq!(back, rows);
    }

    #[test]
    fn parquet_roundtrip_preserves_repo_rows_including_bool() {
        let rows = vec![
            RepoRow {
                repo: "o/r1".into(),
                visibility: "public".into(),
                archived: true,
                pushed_at: "2026-08-01T00:00:00Z".into(),
                size_kb: 1234,
            },
            RepoRow {
                repo: "o/r2".into(),
                visibility: "private".into(),
                archived: false,
                pushed_at: "2026-07-15T00:00:00Z".into(),
                size_kb: 42,
            },
        ];
        let p = temp_path("repos.parquet");
        write_repos(&p, &rows).unwrap();
        let back = read_repos(&p).unwrap();
        assert_eq!(back.len(), 2);
        assert!(back[0].archived);
        assert!(!back[1].archived);
        assert_eq!(back, rows);
    }

    #[test]
    fn parquet_roundtrip_preserves_release_rows() {
        let rows = vec![ReleaseRow {
            repo: "o/r".into(),
            tag: "v1.2.3".into(),
            published_at: "2026-06-01T00:00:00Z".into(),
        }];
        let p = temp_path("releases.parquet");
        write_releases(&p, &rows).unwrap();
        let back = read_releases(&p).unwrap();
        assert_eq!(back.len(), 1);
        assert_eq!(back[0].tag, "v1.2.3");
        assert_eq!(back, rows);
    }
}
