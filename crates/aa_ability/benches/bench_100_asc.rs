use aa_ability::{AaAbilityPlugin, AbilityActivatedEvent, AttributeSet, GameplayAbilityAsset};
use aa_core::AaCorePlugin;
use aa_tags::AaTagsPlugin;
use bevy::prelude::*;
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn asc_bench_world() -> World {
    let mut world = World::new();
    world.add_plugins(AaCorePlugin::default());
    world.add_plugins(AaTagsPlugin);
    world.add_plugins(AaAbilityPlugin);
    world
}

fn bench_100_asc(c: &mut Criterion) {
    c.bench_function("bench_100_asc", |b| {
        b.iter(|| {
            let mut world = asc_bench_world();
            for _ in 0..100 {
                let entity = world.spawn(AttributeSet::default()).id();
                black_box(entity);
                world
                    .resource_mut::<Messages<AbilityActivatedEvent>>()
                    .write(AbilityActivatedEvent {
                        caster: entity,
                        ability_id: "abilities/fireball".into(),
                    });
            }
            black_box(world.entities().len());
        });
    });
}

criterion_group!(benches, bench_100_asc);
criterion_main!(benches);
