use aa_ability::{
    apply_gameplay_effect, grant_ability, AbilityActivatedEvent, AbilityRegistry, AaAbilityPlugin,
    ActiveEffects, AttributeSet, EffectDuration, GameplayAbilityAsset, GameplayEffectAsset,
    ModifierOp,
};
use aa_tags::{GameplayTags, TagRegistry};
use bevy::prelude::*;

fn test_app() -> App {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, AssetPlugin::default(), AaAbilityPlugin));
    app.insert_resource(TagRegistry::default());
    app
}

#[test]
fn effect_modifies_attribute_only_via_gameplay_effect() {
    let mut app = test_app();

    let target = app
        .world_mut()
        .spawn((
            AttributeSet::default(),
            ActiveEffects::default(),
            GameplayTags::default(),
        ))
        .id();

    {
        let mut attrs = app.world_mut().get_mut::<AttributeSet>(target).unwrap();
        attrs.insert_attribute("Health", 100.0, 0.0, 100.0);
    }

    let effect = GameplayEffectAsset {
        schema_version: 1,
        id: "effects/test_damage".into(),
        duration: EffectDuration::Instant,
        modifiers: vec![aa_ability::EffectModifier {
            attribute: "Health".into(),
            op: ModifierOp::Add,
            magnitude: -25.0,
        }],
        granted_tags: Vec::new(),
        application_tags_required: Vec::new(),
        application_tags_blocked: Vec::new(),
        cues_on_apply: Vec::new(),
    };

    let handle = app
        .world_mut()
        .resource_mut::<Assets<GameplayEffectAsset>>()
        .add(effect);

    app.add_systems(Update, move |mut commands: Commands,
                                  effects: Res<Assets<GameplayEffectAsset>>,
                                  tag_registry: Res<TagRegistry>,
                                  mut tags: Query<&mut GameplayTags>,
                                  mut attributes: Query<&mut AttributeSet>,
                                  mut active_effects: Query<&mut ActiveEffects>,
                                  mut cue_writer: MessageWriter<aa_ability::GameplayCueEvent>,
                                  mut damage_writer: MessageWriter<aa_ability::DamageAppliedEvent>,
                                  mut applied: Local<bool>| {
        if *applied {
            return;
        }
        *applied = true;
        let Some(effect_asset) = effects.get(&handle) else {
            return;
        };
        apply_gameplay_effect(
            &mut commands,
            target,
            effect_asset,
            handle.clone(),
            &tag_registry,
            &mut tags,
            &mut attributes,
            &mut active_effects,
            &mut cue_writer,
            &mut damage_writer,
            target,
        );
    });

    app.update();

    let attrs = app.world().get::<AttributeSet>(target).unwrap();
    assert!((attrs.get("Health").unwrap() - 75.0).abs() <= 0.01);
}

#[test]
fn asc_on_player_state_survives_pawn_despawn() {
    let mut app = test_app();

    let player_state = app
        .world_mut()
        .spawn((
            PlayerStateMarker,
            AbilityRegistry::default(),
            AttributeSet::default(),
            GameplayTags::default(),
        ))
        .id();

    let pawn = app.world_mut().spawn(()).id();

    let ability = GameplayAbilityAsset {
        schema_version: 1,
        id: "abilities/fireball".into(),
        display_name: "Fireball".into(),
        cooldown_tags: Vec::new(),
        activation_tags_required: Vec::new(),
        activation_tags_blocked: Vec::new(),
        cost_effect: None,
        montage: None,
        cue_on_activate: None,
        r#impl: "fireball".into(),
    };
    let handle = app
        .world_mut()
        .resource_mut::<Assets<GameplayAbilityAsset>>()
        .add(ability);

    {
        let mut registry = app
            .world_mut()
            .get_mut::<AbilityRegistry>(player_state)
            .unwrap();
        grant_ability(&mut registry, "abilities/fireball", handle);
    }

    app.world_mut().despawn(pawn);
    app.update();

    let registry = app.world().get::<AbilityRegistry>(player_state).unwrap();
    assert_eq!(registry.granted.len(), 1);
}

#[test]
fn stun_blocks_fire_activation() {
    let mut app = test_app();

    let stun_tag = {
        let mut tag_registry = app.world_mut().resource_mut::<TagRegistry>();
        tag_registry.register("State.Stunned")
    };

    let caster = app
        .world_mut()
        .spawn((
            AbilityRegistry::default(),
            AttributeSet::default(),
            GameplayTags::default(),
        ))
        .id();

    {
        let mut tags = app.world_mut().get_mut::<GameplayTags>(caster).unwrap();
        tags.insert(stun_tag);
    }

    let ability = GameplayAbilityAsset {
        schema_version: 1,
        id: "abilities/fireball".into(),
        display_name: "Fireball".into(),
        cooldown_tags: Vec::new(),
        activation_tags_required: Vec::new(),
        activation_tags_blocked: vec!["State.Stunned".into()],
        cost_effect: None,
        montage: None,
        cue_on_activate: None,
        r#impl: "fireball".into(),
    };
    let handle = app
        .world_mut()
        .resource_mut::<Assets<GameplayAbilityAsset>>()
        .add(ability);

    {
        let mut registry = app.world_mut().get_mut::<AbilityRegistry>(caster).unwrap();
        grant_ability(&mut registry, "abilities/fireball", handle);
    }

    app.world_mut()
        .resource_mut::<Messages<AbilityActivatedEvent>>()
        .write(AbilityActivatedEvent {
            caster,
            ability_id: "abilities/fireball".into(),
        });

    app.update();

    let confirmed: Vec<_> = app
        .world_mut()
        .resource_mut::<Messages<aa_ability::AbilityConfirmedEvent>>()
        .drain()
        .collect();
    assert!(confirmed.is_empty(), "stunned caster must not confirm ability");
}

#[derive(Component)]
struct PlayerStateMarker;
