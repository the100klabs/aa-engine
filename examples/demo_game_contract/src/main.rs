const FIREBALL_ABILITY: &str = include_str!("../assets/abilities/fireball.ron");
const FIREBALL_COST_EFFECT: &str = include_str!("../assets/effects/fireball_cost.ron");
const FIREBALL_DAMAGE_EFFECT: &str = include_str!("../assets/effects/fireball_damage.ron");
const TAGS: &str = include_str!("../assets/data/tags.ron");
const COMBATANT_ATTRIBUTES: &str = include_str!("../assets/attributes/combatant.ron");
const PLAYER_COMBAT_INPUT: &str = include_str!("../assets/input/contexts/player_combat.ron");
const PLAYER_MAGE_PAWN: &str = include_str!("../assets/pawns/player_mage.ron");
const TARGET_DUMMY_PAWN: &str = include_str!("../assets/pawns/target_dummy.ron");
const DEMO_ACTION_SET: &str = include_str!("../assets/action_sets/demo_combat.ron");
const DEMO_EXPERIENCE: &str = include_str!("../assets/experiences/demo_combat.ron");
const FIREBALL_HIT_PLAYTEST: &str = include_str!("../assets/playtests/fireball_hit.ron");

fn main() {
    if std::env::args().any(|arg| arg == "--describe") {
        println!(
            "demo_game contract assets: ability={} effects={} tags={} attributes={} input={} pawns={} action_set={} experience={} playtest={}",
            FIREBALL_ABILITY.len(),
            FIREBALL_COST_EFFECT.len() + FIREBALL_DAMAGE_EFFECT.len(),
            TAGS.len(),
            COMBATANT_ATTRIBUTES.len(),
            PLAYER_COMBAT_INPUT.len(),
            PLAYER_MAGE_PAWN.len() + TARGET_DUMMY_PAWN.len(),
            DEMO_ACTION_SET.len(),
            DEMO_EXPERIENCE.len(),
            FIREBALL_HIT_PLAYTEST.len()
        );
    }
}
