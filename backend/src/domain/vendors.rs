use crate::db::{VendorRepo, VendorRow};
use anyhow::anyhow;
use serde::Serialize;

pub async fn get_vendors<R: VendorRepo>(db: &R) -> anyhow::Result<Vendors> {
    let vendors = db.get_vendors().await?;
    let vendors = Vendors::try_from(vendors).map_err(|e| anyhow!(e))?;
    Ok(vendors)
}

#[derive(Serialize)]
pub struct Vendor {
    id: String,
    name: String,
    website: Option<String>,
}

impl TryFrom<VendorRow> for Vendor {
    type Error = &'static str;

    fn try_from(value: VendorRow) -> Result<Self, Self::Error> {
        if value.name.is_empty() {
            return Err("vendor name is empty");
        }

        Ok(Vendor {
            id: value.id.to_string(),
            name: value.name,
            website: value.website,
        })
    }
}

#[derive(Serialize)]
pub struct Vendors {
    vendors: Vec<Vendor>,
}

impl TryFrom<Vec<VendorRow>> for Vendors {
    type Error = &'static str;

    fn try_from(value: Vec<VendorRow>) -> Result<Self, Self::Error> {
        let vendors = value
            .into_iter()
            .map(Vendor::try_from)
            .collect::<Result<Vec<_>, _>>()?;
        Ok(Vendors { vendors })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;
    use uuid::Uuid;

    struct MockVendorRepo {
        rows: Mutex<Option<Vec<VendorRow>>>,
    }

    impl VendorRepo for MockVendorRepo {
        async fn get_vendors(&self) -> anyhow::Result<Vec<VendorRow>> {
            Ok(self.rows.lock().expect("lock poisoned").take().unwrap_or_default())
        }
    }

    fn sample_vendor_row(name: &str) -> VendorRow {
        VendorRow {
            id: Uuid::nil(),
            name: name.to_string(),
            notes: None,
            website: Some("https://example.com".to_string()),
            menu: None,
        }
    }

    #[tokio::test]
    async fn get_vendors_maps_rows() {
        let repo = MockVendorRepo {
            rows: Mutex::new(Some(vec![sample_vendor_row("El Pirata Tortas Y Burritos")])),
        };

        let result = get_vendors(&repo).await.expect("valid result");
        assert_eq!(result.vendors.len(), 1);
        let first = &result.vendors[0];
        assert_eq!(first.id, Uuid::nil().to_string());
        assert_eq!(first.name, "El Pirata Tortas Y Burritos");
    }

    #[test]
    fn vendor_try_from_rejects_empty_name() {
        let row = sample_vendor_row("");
        let result = Vendor::try_from(row);
        assert!(result.is_err());
    }
}
