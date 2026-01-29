use crate::db::{BreweryRepo, BreweryRow};
use anyhow::anyhow;
use serde::Serialize;

pub async fn get_breweries<R: BreweryRepo>(db: &R) -> anyhow::Result<Breweries> {
    let breweries = db.get_breweries().await?;
    let breweries = Breweries::try_from(breweries).map_err(|e| anyhow!(e))?;
    Ok(breweries)
}

#[derive(Serialize)]
pub struct Brewery {
    id: String,
    name: String,
    address: Option<String>,
    lat: Option<f64>,
    lng: Option<f64>,
    website: Option<String>,
}

impl TryFrom<BreweryRow> for Brewery {
    type Error = &'static str;

    fn try_from(value: BreweryRow) -> Result<Self, Self::Error> {
        if value.name.is_empty() {
            return Err("brewery name is empty");
        }

        Ok(Brewery {
            id: value.id.to_string(),
            name: value.name,
            address: value.address,
            lat: value.lat,
            lng: value.lng,
            website: value.website,
        })
    }
}

#[derive(Serialize)]
pub struct Breweries {
    breweries: Vec<Brewery>,
}

impl TryFrom<Vec<BreweryRow>> for Breweries {
    type Error = &'static str;

    fn try_from(value: Vec<BreweryRow>) -> Result<Self, Self::Error> {
        let breweries = value
            .into_iter()
            .map(Brewery::try_from)
            .collect::<Result<Vec<_>, _>>()?;
        Ok(Breweries { breweries })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;
    use uuid::Uuid;

    struct MockBreweryRepo {
        rows: Mutex<Option<Vec<BreweryRow>>>,
    }

    impl BreweryRepo for MockBreweryRepo {
        async fn get_breweries(&self) -> anyhow::Result<Vec<BreweryRow>> {
            Ok(self.rows.lock().expect("lock poisoned").take().unwrap_or_default())
        }
    }

    fn sample_brewery_row(name: &str) -> BreweryRow {
        BreweryRow {
            id: Uuid::nil(),
            name: name.to_string(),
            notes: None,
            website: Some("https://example.com".to_string()),
            address: Some("123 Ballard Ave".to_string()),
            lat: Some(47.6665),
            lng: Some(-122.3711),
            drink_menu: None,
            food_schedule: None,
        }
    }

    #[tokio::test]
    async fn get_breweries_maps_rows() {
        let repo = MockBreweryRepo {
            rows: Mutex::new(Some(vec![sample_brewery_row("Stoup Brewing")])),
        };

        let result = get_breweries(&repo).await.expect("valid result");
        assert_eq!(result.breweries.len(), 1);
        let first = &result.breweries[0];
        assert_eq!(first.id, Uuid::nil().to_string());
        assert_eq!(first.name, "Stoup Brewing");
    }

    #[test]
    fn brewery_try_from_rejects_empty_name() {
        let row = sample_brewery_row("");
        let result = Brewery::try_from(row);
        assert!(result.is_err());
    }
}
