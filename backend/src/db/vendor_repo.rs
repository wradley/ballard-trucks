use crate::db::Db;
use anyhow::Context;
use strsim::{damerau_levenshtein, jaro_winkler};
use uuid::Uuid;

#[derive(sqlx::FromRow)]
pub struct VendorRow {
    pub id: String,
    pub name: String,
    pub website: Option<String>,
}

/// Read access for vendor rows.
#[allow(async_fn_in_trait)]
pub trait VendorRepo {
    async fn get_vendors(&self) -> anyhow::Result<Vec<VendorRow>>;
}

#[allow(async_fn_in_trait)]
pub trait VendorWriteRepo {
    async fn resolve_or_create_vendor_id(
        &self,
        vendor_name: &str,
        normalized_name: &str,
        now_rfc3339: &str,
    ) -> anyhow::Result<String>;
}

impl VendorRepo for Db {
    async fn get_vendors(&self) -> anyhow::Result<Vec<VendorRow>> {
        sqlx::query_as::<_, VendorRow>(
            r#"
    SELECT id, name, website FROM vendors
    ORDER BY name;
                "#,
        )
        .fetch_all(&self.pool)
        .await
        .context("Failed to fetch vendors from SQLite table 'vendors'")
    }
}

impl VendorWriteRepo for Db {
    async fn resolve_or_create_vendor_id(
        &self,
        vendor_name: &str,
        normalized_name: &str,
        now_rfc3339: &str,
    ) -> anyhow::Result<String> {
        let existing_vendor_id: Option<String> =
            sqlx::query_scalar("SELECT id FROM vendors WHERE normalized_name = ? LIMIT 1;")
                .bind(normalized_name)
                .fetch_optional(&self.pool)
                .await
                .context("Failed to resolve vendor by normalized name")?;

        if let Some(id) = existing_vendor_id {
            return Ok(id);
        }

        let candidates = sqlx::query_as::<_, VendorCandidate>(
            r#"
            SELECT id, normalized_name
            FROM vendors
            WHERE normalized_name IS NOT NULL AND normalized_name != '';
            "#,
        )
        .fetch_all(&self.pool)
        .await
        .context("Failed to fetch vendor candidates for fuzzy matching")?;

        if let Some(id) = find_best_vendor_match(normalized_name, &candidates) {
            return Ok(id);
        }

        let id = Uuid::new_v4().to_string();
        sqlx::query(
            r#"
            INSERT INTO vendors (
              id, name, normalized_name, needs_review, match_method, created_at, updated_at
            )
            VALUES (?, ?, ?, 1, 'created', ?, ?);
            "#,
        )
        .bind(&id)
        .bind(vendor_name)
        .bind(normalized_name)
        .bind(now_rfc3339)
        .bind(now_rfc3339)
        .execute(&self.pool)
        .await
        .context("Failed to create vendor during ingest")?;

        Ok(id)
    }
}

#[derive(sqlx::FromRow)]
struct VendorCandidate {
    id: String,
    normalized_name: String,
}

fn find_best_vendor_match(normalized_name: &str, candidates: &[VendorCandidate]) -> Option<String> {
    let mut best: Option<(&VendorCandidate, f64)> = None;

    for candidate in candidates {
        let score = jaro_winkler(normalized_name, &candidate.normalized_name);
        let is_prefix_variant = is_contained_variant(normalized_name, &candidate.normalized_name);
        let is_single_typo = is_single_typo_variant(normalized_name, &candidate.normalized_name);

        if score >= 0.95 || is_prefix_variant || is_single_typo {
            if best.is_none_or(|(_, current)| score > current) {
                best = Some((candidate, score));
            }
        }
    }

    best.map(|(candidate, _)| candidate.id.clone())
}

fn is_contained_variant(a: &str, b: &str) -> bool {
    let (shorter, longer) = if a.len() <= b.len() { (a, b) } else { (b, a) };
    shorter.len() >= 6 && longer.len() - shorter.len() <= 6 && longer.contains(shorter)
}

fn is_single_typo_variant(a: &str, b: &str) -> bool {
    let min_len = a.len().min(b.len());
    min_len >= 8 && damerau_levenshtein(a, b) <= 1
}

#[cfg(test)]
mod tests {
    use super::{VendorCandidate, find_best_vendor_match};

    #[test]
    fn fuzzy_match_catches_common_vendor_variants() {
        let candidates = vec![
            VendorCandidate {
                id: "a".to_string(),
                normalized_name: "tacoscalifas".to_string(),
            },
            VendorCandidate {
                id: "b".to_string(),
                normalized_name: "kaosamai".to_string(),
            },
            VendorCandidate {
                id: "c".to_string(),
                normalized_name: "impeccablechicken".to_string(),
            },
        ];

        assert_eq!(
            find_best_vendor_match("tacoscalifasdebut", &candidates).as_deref(),
            Some("a")
        );
        assert_eq!(
            find_best_vendor_match("kaosamaithai", &candidates).as_deref(),
            Some("b")
        );
        assert_eq!(
            find_best_vendor_match("impeckablechicken", &candidates).as_deref(),
            Some("c")
        );
    }

    #[test]
    fn fuzzy_match_does_not_merge_distinct_vendors() {
        let candidates = vec![
            VendorCandidate {
                id: "a".to_string(),
                normalized_name: "tacoscalifas".to_string(),
            },
            VendorCandidate {
                id: "b".to_string(),
                normalized_name: "impeckablechicken".to_string(),
            },
            VendorCandidate {
                id: "c".to_string(),
                normalized_name: "impeccablechicken".to_string(),
            },
        ];

        assert_eq!(find_best_vendor_match("californiatacos", &candidates), None);
        assert_eq!(
            find_best_vendor_match("impossiblechicken", &candidates),
            None
        );
        assert_eq!(find_best_vendor_match("impeccablebeef", &candidates), None);
    }
}
