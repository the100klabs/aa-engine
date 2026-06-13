const WORLD: &str = include_str!("../assets/worlds/open_world_studio.ron");
const SECTOR_0_0: &str = include_str!("../assets/sectors/sector_0_0.ron");
const ENEMY_CAMP: &str = include_str!("../assets/spawn_tables/enemy_camp_sector_0_0.ron");
const CAMP_GUARD_AI: &str = include_str!("../assets/ai/camp_guard.ron");
const ENEMY_CAMP_PLAYTEST: &str = include_str!("../assets/playtests/open_world_enemy_camp.ron");
const CAMP_GUARD_PAWN: &str = include_str!("../assets/pawns/camp_guard.ron");
const PLAYER_EXPLORER_PAWN: &str = include_str!("../assets/pawns/player_explorer.ron");
const CAMP_FIRE_PREFAB: &str = include_str!("../assets/prefabs/camp_fire.ron");
const CAMP_GUARD_SPAWN_PREFAB: &str = include_str!("../assets/prefabs/camp_guard_spawn.ron");
const BASIC_MELEE_ABILITY: &str = include_str!("../assets/abilities/basic_melee.ron");
const CAMP_GUARD_BASELINE_EFFECT: &str = include_str!("../assets/effects/camp_guard_baseline.ron");
const TAGS: &str = include_str!("../assets/data/tags.ron");

fn main() {
    if std::env::args().any(|arg| arg == "--describe") {
        println!(
            "open_world_studio contract assets: world={} sector={} spawn_table={} ai={} playtest={} pawns={} prefabs={} ability={} effect={} tags={}",
            WORLD.len(),
            SECTOR_0_0.len(),
            ENEMY_CAMP.len(),
            CAMP_GUARD_AI.len(),
            ENEMY_CAMP_PLAYTEST.len(),
            CAMP_GUARD_PAWN.len() + PLAYER_EXPLORER_PAWN.len(),
            CAMP_FIRE_PREFAB.len() + CAMP_GUARD_SPAWN_PREFAB.len(),
            BASIC_MELEE_ABILITY.len(),
            CAMP_GUARD_BASELINE_EFFECT.len(),
            TAGS.len()
        );
    }
}
