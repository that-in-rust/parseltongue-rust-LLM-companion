use parseltongue_core_row_types::StageRunScopeValue;

pub(super) async fn resolve_snapshot_read_scope(
    connection: &turso::Connection,
    scope: &StageRunScopeValue,
) -> Result<(StageRunScopeValue, Option<i64>), turso::Error> {
    let mut rows=connection.query("SELECT source_attempt_key,publication_revision FROM \"frozen-stage-publication-records\"
        WHERE generation_key=? AND attempt_key=? ORDER BY generation_key,attempt_key",
        turso::params![scope.run_generation_digest_value.clone(),scope.stage_attempt_sequence_value.clone()]).await?;
    if let Some(row) = rows.next().await? {
        Ok((
            StageRunScopeValue {
                run_generation_digest_value: scope.run_generation_digest_value.clone(),
                stage_attempt_sequence_value: row.get(0)?,
            },
            Some(row.get(1)?),
        ))
    } else {
        Ok((scope.clone(), None))
    }
}

pub(super) fn build_snapshot_unit_visibility(table: &str, revision: Option<i64>) -> String {
    let Some(revision) = revision else {
        return "1".into();
    };
    let marker = if table == "stage-run-generation-records" {
        format!("\"{table}\".stage_row_record_key")
    } else {
        let owner = super::STAGE_OWNED_TABLE_NAMES
            .iter()
            .find(|(name, _)| *name == table)
            .map(|(_, owner)| *owner)
            .unwrap_or("unknown");
        if owner == "syntax-spans"
            || owner == "source-ingest"
            || owner == "syntax-namespace-input-records"
        {
            let build_frozen_owner_probe = |owner: &str| {
                format!("EXISTS (SELECT 1 FROM \"frozen-stage-unit-memberships\" AS frozen WHERE frozen.generation_key=\"{table}\".run_generation_digest_value
                AND frozen.source_attempt_key=\"{table}\".stage_attempt_sequence_value AND frozen.publication_revision<={revision}
                AND frozen.stage_marker_key='__marker__/{owner}/' || \"{table}\".stage_unit_work_key)")
            };
            return format!(
                "({} OR {})",
                build_frozen_owner_probe(owner),
                build_frozen_owner_probe("name-resolution")
            );
        }
        format!("'__marker__/{owner}/' || \"{table}\".stage_unit_work_key")
    };
    format!("EXISTS (SELECT 1 FROM \"frozen-stage-unit-memberships\" AS frozen WHERE frozen.generation_key=\"{table}\".run_generation_digest_value
        AND frozen.source_attempt_key=\"{table}\".stage_attempt_sequence_value AND frozen.publication_revision<={revision}
        AND frozen.stage_marker_key={marker})")
}

pub(super) async fn seal_snapshot_publication(
    connection: &turso::Connection,
    scope: &StageRunScopeValue,
) -> Result<StageRunScopeValue, turso::Error> {
    let mut rows=connection.query("SELECT coalesce(max(publication_revision),0)+1 FROM \"frozen-stage-publication-records\"
        WHERE generation_key=? AND source_attempt_key=?",
        turso::params![scope.run_generation_digest_value.clone(),scope.stage_attempt_sequence_value.clone()]).await?;
    let revision: i64 = rows
        .next()
        .await?
        .ok_or(turso::Error::QueryReturnedNoRows)?
        .get(0)?;
    drop(rows);
    let attempt = if revision == 1 {
        scope.stage_attempt_sequence_value.clone()
    } else {
        format!("{}/frozen/{revision}", scope.stage_attempt_sequence_value)
    };
    connection.execute("INSERT INTO \"frozen-stage-unit-memberships\"
        SELECT run_generation_digest_value,stage_attempt_sequence_value,stage_row_record_key,?
        FROM \"stage-run-generation-records\" AS markers WHERE run_generation_digest_value=? AND stage_attempt_sequence_value=?
        AND NOT EXISTS (SELECT 1 FROM \"frozen-stage-unit-memberships\" frozen WHERE frozen.generation_key=markers.run_generation_digest_value
            AND frozen.source_attempt_key=markers.stage_attempt_sequence_value AND frozen.stage_marker_key=markers.stage_row_record_key)",
        turso::params![revision,scope.run_generation_digest_value.clone(),scope.stage_attempt_sequence_value.clone()]).await?;
    connection
        .execute(
            "INSERT INTO \"frozen-stage-publication-records\" VALUES (?,?,?,?)",
            turso::params![
                scope.run_generation_digest_value.clone(),
                attempt.clone(),
                scope.stage_attempt_sequence_value.clone(),
                revision
            ],
        )
        .await?;
    Ok(StageRunScopeValue {
        run_generation_digest_value: scope.run_generation_digest_value.clone(),
        stage_attempt_sequence_value: attempt,
    })
}
