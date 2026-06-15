//! P3-06 proxy — sector scene RON round-trip preserves entity placements.

use std::path::Path;

use aa_world_stream::{
    load_sector_descriptor_from_disk, SectorDescriptorAsset, SectorEntityPlacement, WorldBounds,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename = "SectorDescriptor")]
struct SectorRoundTripRon {
    schema_version: u32,
    id: String,
    coord: [i32; 2],
    bounds: WorldBounds,
    data_layers: Vec<String>,
    entities: Vec<SectorEntityPlacement>,
}

fn to_round_trip(asset: &SectorDescriptorAsset) -> SectorRoundTripRon {
    SectorRoundTripRon {
        schema_version: asset.schema_version,
        id: asset.id.clone(),
        coord: asset.coord,
        bounds: asset.bounds.clone(),
        data_layers: asset.data_layers.clone(),
        entities: asset.entities.clone(),
    }
}

#[test]
fn scene_ron_round_trip_sector_entity_count() {
    let project = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../examples/open_world_studio");
    let original =
        load_sector_descriptor_from_disk(&project, "assets/sectors/sector_0_0.ron")
            .expect("load sector_0_0");

    let ron_text = ron::ser::to_string(&to_round_trip(&original)).expect("serialize sector");
    let round_tripped: SectorRoundTripRon =
        ron::from_str(&ron_text).expect("deserialize round-tripped sector");

    assert_eq!(round_tripped.id, original.id);
    assert_eq!(round_tripped.entities.len(), original.entities.len());
    for (left, right) in original.entities.iter().zip(round_tripped.entities.iter()) {
        assert_eq!(left.prefab, right.prefab);
        assert_eq!(left.transform.translation, right.transform.translation);
        assert_eq!(
            left.transform.rotation_y_degrees,
            right.transform.rotation_y_degrees
        );
        assert_eq!(left.transform.scale, right.transform.scale);
    }
}
