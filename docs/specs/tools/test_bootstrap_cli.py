#!/usr/bin/env python3
"""Tests for docs/specs/tools/aa_bootstrap.py."""

from __future__ import annotations

import json
import shutil
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))

from schema_subset import load_json, validate_schema


REPO_ROOT = Path(__file__).resolve().parents[3]
BOOTSTRAP = REPO_ROOT / "docs/specs/tools/aa_bootstrap.py"
SHIM = REPO_ROOT / "aa"
SCHEMAS = REPO_ROOT / "docs/specs/schemas"


PROJECT_TOML = """\
schema_version = 1
name = "fixture_game"
version = "0.1.0"

[engine]
config_root = "project_config"
assets_root = "game_assets"

[plugins]
runtime = []

[features]
enabled = []

[build]
default_binary = "fixture_game"
"""

ENGINE_TOML = """\
[app]
window_title = "Fixture"
target_fps = 60

[render]
resolution_scale = 1.0
msaa = 4
shadow_quality = "high"
gi = "probes"

[audio]
master_volume = 1.0

[net]
default_port = 7777
max_players = 8
"""

GAME_TOML = """\
[gameplay]
default_damage_scale = 1.0
respawn_delay_secs = 3.0

[teams]
max_teams = 2
friendly_fire = false
"""

INPUT_TOML = """\
[defaults]
mouse_sensitivity = 1.0
gamepad_deadzone = 0.15
"""

SCALABILITY_TOML = """\
[presets.low.render]
resolution_scale = 0.5
shadow_quality = "low"
gi = "off"

[presets.medium.render]
resolution_scale = 0.75
shadow_quality = "medium"
gi = "baked"

[presets.high.render]
resolution_scale = 1.0
shadow_quality = "high"
gi = "probes"

[presets.epic.render]
resolution_scale = 1.0
shadow_quality = "epic"
gi = "probes_plus"
"""


def write_valid_project(root: Path) -> None:
    (root / "aa.project.toml").write_text(PROJECT_TOML, encoding="utf-8")
    config = root / "project_config"
    config.mkdir()
    (root / "game_assets").mkdir()
    (config / "engine.toml").write_text(ENGINE_TOML, encoding="utf-8")
    (config / "game.toml").write_text(GAME_TOML, encoding="utf-8")
    (config / "input.toml").write_text(INPUT_TOML, encoding="utf-8")
    (config / "scalability.toml").write_text(SCALABILITY_TOML, encoding="utf-8")


def write_cargo_project(root: Path, source: str) -> None:
    (root / "Cargo.toml").write_text(
        """\
[package]
name = "fixture_check"
version = "0.1.0"
edition = "2021"
""",
        encoding="utf-8",
    )
    src = root / "src"
    src.mkdir()
    (src / "lib.rs").write_text(source, encoding="utf-8")


def write_project_with_unknown_effect_attribute(root: Path) -> None:
    write_valid_project(root)
    attributes = root / "game_assets/attributes"
    effects = root / "game_assets/effects"
    attributes.mkdir()
    effects.mkdir()
    (attributes / "combatant.ron").write_text(
        """\
(
    schema_version: 1,
    id: "combatant",
    attributes: [
        (
            name: "Health",
            default: 100.0,
            min: 0.0,
            max: 100.0,
            replicated: true,
        ),
    ],
)
""",
        encoding="utf-8",
    )
    (effects / "bad_stamina.ron").write_text(
        """\
(
    schema_version: 1,
    id: "bad_stamina",
    duration: "Instant",
    modifiers: [
        (
            attribute: "Stamina",
            op: "Add",
            magnitude: -10.0,
        ),
    ],
    granted_tags: [],
    application_tags_required: [],
    application_tags_blocked: [],
    cues_on_apply: [],
)
""",
        encoding="utf-8",
    )


def run_bootstrap(*args: str) -> subprocess.CompletedProcess[str]:
    return subprocess.run(
        ["python3", str(BOOTSTRAP), *args],
        cwd=REPO_ROOT,
        text=True,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        check=False,
    )


def run_shim(*args: str) -> subprocess.CompletedProcess[str]:
    return subprocess.run(
        [str(SHIM), *args],
        cwd=REPO_ROOT,
        text=True,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        check=False,
    )


def assert_schema_valid(testcase: unittest.TestCase, schema_name: str, payload: dict) -> None:
    schema = load_json(SCHEMAS / schema_name)
    try:
        validate_schema(schema, payload, schema, "$")
    except Exception as exc:  # pragma: no cover - unittest prints payload context below.
        testcase.fail(f"{schema_name} validation failed: {exc}\npayload={json.dumps(payload, indent=2)}")


def assert_sarif_valid(testcase: unittest.TestCase, payload: dict) -> None:
    schema = load_json(SCHEMAS / "sarif_2_1_min.schema.json")
    try:
        validate_schema(schema, payload, schema, "$")
    except Exception as exc:  # pragma: no cover - unittest prints payload context below.
        testcase.fail(f"sarif_2_1_min.schema.json validation failed: {exc}\npayload={json.dumps(payload, indent=2)}")


class BootstrapCliTests(unittest.TestCase):
    def test_validate_good_project_uses_manifest_roots(self) -> None:
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            write_valid_project(root)

            result = run_bootstrap("validate", str(root), "--format", "json")
            self.assertEqual(result.returncode, 0, result.stderr)
            payload = json.loads(result.stdout)

        self.assertTrue(payload["ok"])
        self.assertEqual(payload["diagnostics"], [])
        assert_schema_valid(self, "validation_result.schema.json", payload)

    def test_project_local_shim_delegates_to_bootstrap_validate(self) -> None:
        result = run_shim("validate", "--format", "json")
        self.assertEqual(result.returncode, 0, result.stderr)
        payload = json.loads(result.stdout)

        self.assertTrue(payload["ok"])
        self.assertEqual(payload["diagnostics"], [])
        assert_schema_valid(self, "validation_result.schema.json", payload)

    def test_validate_initialized_empty_project_fixture(self) -> None:
        result = run_bootstrap("validate", "docs/specs/fixtures/empty_project", "--format", "json")
        self.assertEqual(result.returncode, 0, result.stderr)
        payload = json.loads(result.stdout)

        self.assertTrue(payload["ok"])
        self.assertEqual(payload["diagnostics"], [])
        assert_schema_valid(self, "validation_result.schema.json", payload)

    def test_validate_sarif_good_project_has_no_results(self) -> None:
        result = run_bootstrap("validate", "examples/demo_game_contract", "--format", "sarif")
        self.assertEqual(result.returncode, 0, result.stderr)
        payload = json.loads(result.stdout)

        self.assertEqual(payload["version"], "2.1.0")
        self.assertEqual(payload["runs"][0]["tool"]["driver"]["name"], "aa validate")
        self.assertEqual(payload["runs"][0]["results"], [])
        assert_sarif_valid(self, payload)

    def test_validate_uninitialized_directory_reports_missing_manifest(self) -> None:
        with tempfile.TemporaryDirectory() as temp:
            result = run_bootstrap("validate", temp, "--format", "json")
            self.assertEqual(result.returncode, 1)
            payload = json.loads(result.stdout)

            self.assertFalse(payload["ok"])
            self.assertEqual(payload["diagnostics"][0]["code"], "FILE_MISSING")
            self.assertEqual(payload["diagnostics"][0]["path"], "aa.project.toml")
            assert_schema_valid(self, "validation_result.schema.json", payload)

    def test_validate_reports_project_relative_missing_assets_root(self) -> None:
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            write_valid_project(root)
            (root / "game_assets").rmdir()

            result = run_bootstrap("validate", str(root), "--format", "json")
            self.assertEqual(result.returncode, 1)
            payload = json.loads(result.stdout)

            self.assertFalse(payload["ok"])
            self.assertEqual(payload["error_count"], 1)
            self.assertEqual(payload["diagnostics"][0]["code"], "DIR_MISSING")
            self.assertEqual(payload["diagnostics"][0]["path"], "game_assets")
            assert_schema_valid(self, "validation_result.schema.json", payload)

    def test_validate_sarif_reports_project_relative_diagnostic(self) -> None:
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            write_valid_project(root)
            (root / "game_assets").rmdir()

            result = run_bootstrap("validate", str(root), "--format", "sarif")
            self.assertEqual(result.returncode, 1)
            payload = json.loads(result.stdout)

            results = payload["runs"][0]["results"]
            self.assertEqual(len(results), 1)
            self.assertEqual(results[0]["ruleId"], "DIR_MISSING")
            self.assertEqual(results[0]["level"], "error")
            location = results[0]["locations"][0]["physicalLocation"]["artifactLocation"]
            self.assertEqual(location["uri"], "game_assets")
            rules = payload["runs"][0]["tool"]["driver"]["rules"]
            self.assertEqual(rules[0]["id"], "DIR_MISSING")
            assert_sarif_valid(self, payload)

    def test_validate_reports_missing_spawn_table_refs(self) -> None:
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            write_valid_project(root)
            spawn_tables = root / "game_assets/spawn_tables"
            spawn_tables.mkdir()
            (spawn_tables / "broken_camp.ron").write_text(
                """\
(
    schema_version: 1,
    id: "broken_camp",
    entries: [
        (
            id: "missing_guard",
            pawn: "assets/pawns/missing_guard.ron",
            ai_profile: "assets/ai/missing_guard.ron",
            prefab: "assets/prefabs/missing_guard.ron",
            weight: 1.0,
        ),
    ],
)
""",
                encoding="utf-8",
            )

            result = run_bootstrap("validate", str(root), "--format", "json")
            self.assertEqual(result.returncode, 1)
            payload = json.loads(result.stdout)

            self.assertFalse(payload["ok"])
            codes = [item["code"] for item in payload["diagnostics"]]
            self.assertEqual(codes.count("REF_MISSING"), 3)
            messages = "\n".join(item["message"] for item in payload["diagnostics"])
            self.assertIn("entries[].pawn", messages)
            self.assertIn("entries[].ai_profile", messages)
            self.assertIn("entries[].prefab", messages)
            assert_schema_valid(self, "validation_result.schema.json", payload)

    def test_validate_reports_unknown_effect_modifier_attribute(self) -> None:
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            write_project_with_unknown_effect_attribute(root)

            result = run_bootstrap("validate", str(root), "--format", "json")
            self.assertEqual(result.returncode, 1)
            payload = json.loads(result.stdout)

            self.assertFalse(payload["ok"])
            diagnostics = [item for item in payload["diagnostics"] if item["code"] == "ATTR_UNKNOWN"]
            self.assertEqual(len(diagnostics), 1)
            self.assertEqual(diagnostics[0]["path"], "game_assets/effects/bad_stamina.ron")
            self.assertIn("Stamina", diagnostics[0]["message"])
            assert_schema_valid(self, "validation_result.schema.json", payload)

    def test_validate_sarif_reports_unknown_effect_modifier_attribute(self) -> None:
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            write_project_with_unknown_effect_attribute(root)

            result = run_bootstrap("validate", str(root), "--format", "sarif")
            self.assertEqual(result.returncode, 1)
            payload = json.loads(result.stdout)

            sarif_results = payload["runs"][0]["results"]
            attr_result = next(item for item in sarif_results if item["ruleId"] == "ATTR_UNKNOWN")
            self.assertEqual(attr_result["level"], "error")
            self.assertEqual(
                attr_result["locations"][0]["physicalLocation"]["artifactLocation"]["uri"],
                "game_assets/effects/bad_stamina.ron",
            )
            self.assertIn("Stamina", attr_result["message"]["text"])
            assert_sarif_valid(self, payload)

    def test_index_enemy_camp_result_matches_schema(self) -> None:
        result = run_bootstrap("index", "--query", "enemy camp sector", "--json")
        self.assertEqual(result.returncode, 0, result.stderr)
        payload = json.loads(result.stdout)

        kinds = {hit["kind"] for hit in payload["hits"]}
        self.assertTrue({"spec", "schema", "playtest", "eval", "asset"}.issubset(kinds))
        paths = {hit["path"] for hit in payload["hits"]}
        self.assertIn(
            "examples/open_world_studio/assets/spawn_tables/enemy_camp_sector_0_0.ron",
            paths,
        )
        assert_schema_valid(self, "index_result.schema.json", payload)

    def test_index_open_world_scope_returns_editable_assets(self) -> None:
        result = run_bootstrap(
            "index",
            "--query",
            "camp guard spawn table basic melee",
            "--scope",
            "examples/open_world_studio/assets",
            "--json",
        )
        self.assertEqual(result.returncode, 0, result.stderr)
        payload = json.loads(result.stdout)

        asset_paths = {hit["path"] for hit in payload["hits"] if hit["kind"] == "asset"}
        self.assertTrue(
            {
                "examples/open_world_studio/assets/spawn_tables/enemy_camp_sector_0_0.ron",
                "examples/open_world_studio/assets/ai/camp_guard.ron",
                "examples/open_world_studio/assets/abilities/basic_melee.ron",
                "examples/open_world_studio/assets/pawns/camp_guard.ron",
                "examples/open_world_studio/assets/effects/camp_guard_baseline.ron",
            }.issubset(asset_paths)
        )
        assert_schema_valid(self, "index_result.schema.json", payload)

    def test_index_demo_game_fire_ability_returns_editable_assets(self) -> None:
        result = run_bootstrap(
            "index",
            "--query",
            "fire ability fireball demo_game",
            "--json",
        )
        self.assertEqual(result.returncode, 0, result.stderr)
        payload = json.loads(result.stdout)

        paths = {hit["path"] for hit in payload["hits"]}
        self.assertIn("examples/demo_game_contract/assets/abilities/fireball.ron", paths)
        self.assertIn("examples/demo_game_contract/assets/playtests/fireball_hit.ron", paths)
        self.assertIn("docs/specs/fixtures/demo_game/add_fire_ability.eval.json", paths)
        assert_schema_valid(self, "index_result.schema.json", payload)

    def test_config_get_reads_project_config_value(self) -> None:
        result = run_bootstrap("config", "get", "net.default_port", "--project", "examples/demo_game_contract", "--json")
        self.assertEqual(result.returncode, 0, result.stderr)
        payload = json.loads(result.stdout)

        self.assertTrue(payload["ok"])
        self.assertEqual(payload["project"], "examples/demo_game_contract")
        self.assertEqual(payload["key"], "net.default_port")
        self.assertEqual(payload["value"], 7777)
        self.assertEqual(payload["value_type"], "int")
        self.assertEqual(payload["source"], "config/engine.toml")
        assert_schema_valid(self, "config_get_result.schema.json", payload)

    def test_config_get_reports_missing_key(self) -> None:
        result = run_bootstrap("config", "get", "net.nope", "--project", "examples/demo_game_contract", "--json")
        self.assertEqual(result.returncode, 1)
        payload = json.loads(result.stdout)

        self.assertFalse(payload["ok"])
        self.assertEqual(payload["key"], "net.nope")
        self.assertEqual(payload["source"], "")
        self.assertTrue(any(item["code"] == "CONFIG_KEY_NOT_FOUND" for item in payload["diagnostics"]))
        assert_schema_valid(self, "config_get_result.schema.json", payload)

    def test_eval_list_returns_known_studio_suites(self) -> None:
        result = run_bootstrap("eval", "list", "--json")
        self.assertEqual(result.returncode, 0, result.stderr)
        payload = json.loads(result.stdout)

        self.assertTrue(payload["ok"])
        suite_ids = {suite["id"] for suite in payload["suites"]}
        self.assertEqual(
            suite_ids,
            {
                "demo_game_add_fire_ability",
                "open_world_studio_enemy_camp",
                "open_world_studio_elemental_ability",
            },
        )
        fire = next(suite for suite in payload["suites"] if suite["id"] == "demo_game_add_fire_ability")
        open_world = next(suite for suite in payload["suites"] if suite["id"] == "open_world_studio_enemy_camp")
        elemental = next(
            suite for suite in payload["suites"] if suite["id"] == "open_world_studio_elemental_ability"
        )
        self.assertEqual(fire["tier"], "studio_alpha")
        self.assertEqual(fire["categories"], ["ability"])
        self.assertIn("aa ability graph", fire["required_commands"])
        self.assertEqual(open_world["tier"], "open_world_alpha")
        self.assertEqual(open_world["categories"], ["enemy_camp"])
        self.assertIn("aa world inspect", open_world["required_commands"])
        self.assertEqual(elemental["tier"], "open_world_alpha")
        self.assertEqual(elemental["categories"], ["ability"])
        self.assertIn("aa playtest", elemental["required_commands"])
        assert_schema_valid(self, "eval_list_result.schema.json", payload)

    def test_validate_demo_game_contract_project(self) -> None:
        result = run_bootstrap("validate", "examples/demo_game_contract", "--format", "json")
        self.assertEqual(result.returncode, 0, result.stderr)
        payload = json.loads(result.stdout)

        self.assertTrue(payload["ok"])
        self.assertEqual(payload["diagnostics"], [])
        assert_schema_valid(self, "validation_result.schema.json", payload)

    def test_eval_open_world_fixture_passes_contract_loop(self) -> None:
        result = run_bootstrap("eval", "run", "open_world_studio_enemy_camp", "--json")
        self.assertEqual(result.returncode, 0, result.stdout)
        payload = json.loads(result.stdout)

        self.assertTrue(payload["ok"])
        self.assertEqual(payload["suite"], "open_world_studio_enemy_camp")
        self.assertEqual(payload["pass_rate"], 1.0)
        self.assertTrue(payload["tasks"])
        task = payload["tasks"][0]
        self.assertTrue(task["passed"])
        command_names = {item["command"] for item in task["commands"]}
        self.assertIn("aa index", command_names)
        self.assertIn("aa scene inspect", command_names)
        self.assertIn("aa scene patch", command_names)
        self.assertIn("aa world inspect", command_names)
        validate = next(item for item in task["commands"] if item["command"] == "aa validate")
        check = next(item for item in task["commands"] if item["command"] == "aa check")
        scene_inspect = next(item for item in task["commands"] if item["command"] == "aa scene inspect")
        scene_patch = next(item for item in task["commands"] if item["command"] == "aa scene patch")
        world_inspect = next(item for item in task["commands"] if item["command"] == "aa world inspect")
        world_cook = next(item for item in task["commands"] if item["command"] == "aa world cook")
        playtest = next(item for item in task["commands"] if item["command"] == "aa playtest")
        profile = next(item for item in task["commands"] if item["command"] == "aa profile summarize")
        self.assertEqual(validate["exit_code"], 0)
        self.assertEqual(check["exit_code"], 0)
        self.assertEqual(scene_inspect["exit_code"], 0)
        self.assertEqual(scene_patch["exit_code"], 0)
        self.assertEqual(world_inspect["exit_code"], 0)
        self.assertEqual(world_cook["exit_code"], 0)
        self.assertEqual(playtest["exit_code"], 0)
        self.assertEqual(profile["exit_code"], 0)
        self.assertNotIn("failure_reason", task)
        acceptance = {item["name"]: item for item in task["acceptance"]}
        self.assertTrue(acceptance["ExpectedFile:examples/open_world_studio/assets/worlds/open_world_studio.ron"]["passed"])
        self.assertTrue(acceptance["ForbiddenPathAbsent:target"]["passed"])
        self.assertTrue(acceptance["CommandPasses:aa scene inspect"]["passed"])
        self.assertTrue(acceptance["CommandPasses:aa scene patch"]["passed"])
        self.assertTrue(acceptance["FileChanged:examples/open_world_studio/assets/spawn_tables/enemy_camp_sector_0_0.ron"]["passed"])
        self.assertTrue(acceptance["ProfileBudgetWithin:sector_load_p95_ms"]["passed"])
        self.assertGreaterEqual(len(acceptance), 10)
        assert_schema_valid(self, "eval_report.schema.json", payload)

    def test_eval_open_world_fixture_fails_for_forbidden_target_dir(self) -> None:
        target_dir = REPO_ROOT / "examples/open_world_studio/target"
        target_dir.mkdir(exist_ok=True)
        try:
            result = run_bootstrap("eval", "run", "open_world_studio_enemy_camp", "--json")
            self.assertEqual(result.returncode, 3)
            payload = json.loads(result.stdout)

            self.assertFalse(payload["ok"])
            task = payload["tasks"][0]
            self.assertFalse(task["passed"])
            self.assertIn("forbidden path present: target", task["failure_reason"])
            acceptance = {item["name"]: item for item in task["acceptance"]}
            self.assertFalse(acceptance["ForbiddenPathAbsent:target"]["passed"])
            self.assertIn("target", acceptance["ForbiddenPathAbsent:target"]["message"])
            assert_schema_valid(self, "eval_report.schema.json", payload)
        finally:
            shutil.rmtree(target_dir, ignore_errors=True)

    def test_eval_demo_game_fire_ability_fixture_passes_contract_loop(self) -> None:
        result = run_bootstrap("eval", "run", "demo_game_add_fire_ability", "--json")
        self.assertEqual(result.returncode, 0, result.stdout)
        payload = json.loads(result.stdout)

        self.assertTrue(payload["ok"])
        self.assertEqual(payload["suite"], "demo_game_add_fire_ability")
        self.assertEqual(payload["pass_rate"], 1.0)
        task = payload["tasks"][0]
        self.assertTrue(task["passed"])
        command_names = {item["command"] for item in task["commands"]}
        self.assertTrue({"aa index", "aa validate", "aa check", "aa ability graph", "aa playtest"}.issubset(command_names))
        graph = next(item for item in task["commands"] if item["command"] == "aa ability graph")
        playtest = next(item for item in task["commands"] if item["command"] == "aa playtest")
        self.assertEqual(graph["args"][0], "fireball")
        self.assertIn("fireball_hit", playtest["args"])
        acceptance = {item["name"]: item for item in task["acceptance"]}
        self.assertTrue(acceptance["PlaytestPasses:fireball_hit"]["passed"])
        self.assertTrue(acceptance["FileChanged:examples/demo_game_contract/assets/abilities/fireball.ron"]["passed"])
        self.assertTrue(acceptance["FileChanged:examples/demo_game_contract/assets/effects/fireball_damage.ron"]["passed"])
        assert_schema_valid(self, "eval_report.schema.json", payload)

    def test_eval_open_world_elemental_ability_fixture_passes(self) -> None:
        result = run_bootstrap("eval", "run", "open_world_studio_elemental_ability", "--json")
        self.assertEqual(result.returncode, 0, result.stdout)
        payload = json.loads(result.stdout)

        self.assertTrue(payload["ok"])
        self.assertEqual(payload["suite"], "open_world_studio_elemental_ability")
        self.assertEqual(payload["pass_rate"], 1.0)
        task = payload["tasks"][0]
        self.assertTrue(task["passed"])
        command_names = {item["command"] for item in task["commands"]}
        self.assertTrue({"aa index", "aa validate", "aa playtest"}.issubset(command_names))
        playtest = next(item for item in task["commands"] if item["command"] == "aa playtest")
        self.assertIn("open_world_enemy_camp", playtest["args"])
        acceptance = {item["name"]: item for item in task["acceptance"]}
        self.assertTrue(acceptance["PlaytestPasses:open_world_enemy_camp"]["passed"])
        self.assertTrue(
            acceptance[
                "ExpectedFile:examples/open_world_studio/assets/abilities/basic_ranged_attack.ron"
            ]["passed"]
        )
        assert_schema_valid(self, "eval_report.schema.json", payload)

    def test_world_profile_and_playtest_fixture_outputs_match_schemas(self) -> None:
        commands = [
            (
                ("world", "inspect", "--world", "open_world_studio", "--json"),
                "world_inspect_result.schema.json",
            ),
            (
                ("world", "cook", "--world", "open_world_studio", "--verify", "--json"),
                "world_cook_result.schema.json",
            ),
            (
                (
                    "world",
                    "generate",
                    "--template",
                    "starter_open_world",
                    "--output",
                    "examples/generated_world_preview",
                    "--name",
                    "test_world",
                    "--dry-run",
                    "--json",
                ),
                "world_generate_result.schema.json",
            ),
            (
                ("profile", "summarize", "artifacts/profiles/open_world_enemy_camp.trace", "--json"),
                "profile_summary_result.schema.json",
            ),
            (
                ("playtest", "--scenario", "open_world_enemy_camp", "--json"),
                "playtest_result.schema.json",
            ),
            (
                ("playtest", "--scenario", "smoke", "--json"),
                "playtest_result.schema.json",
            ),
            (
                ("playtest", "--scenario", "fireball_hit", "--json"),
                "playtest_result.schema.json",
            ),
            (
                ("ability", "graph", "basic_melee", "--project", "examples/open_world_studio", "--json"),
                "ability_graph_result.schema.json",
            ),
            (
                ("ability", "graph", "fireball", "--project", "examples/demo_game_contract", "--json"),
                "ability_graph_result.schema.json",
            ),
            (
                (
                    "scene",
                    "list",
                    "--scene",
                    "examples/open_world_studio/assets/sectors/sector_0_0.ron",
                    "--json",
                ),
                "scene_list_result.schema.json",
            ),
            (
                (
                    "scene",
                    "inspect",
                    "sector_0_0/entity_0",
                    "--scene",
                    "examples/open_world_studio/assets/sectors/sector_0_0.ron",
                    "--json",
                ),
                "scene_inspect_result.schema.json",
            ),
            (
                (
                    "scene",
                    "patch",
                    "--scene",
                    "examples/open_world_studio/assets/sectors/sector_0_0.ron",
                    "--patch",
                    "docs/specs/fixtures/open_world_studio/add_campfire.scene_patch.json",
                    "--dry-run",
                    "--json",
                ),
                "scene_patch_result.schema.json",
            ),
        ]
        for args, schema_name in commands:
            with self.subTest(command=args):
                result = run_bootstrap(*args)
                self.assertEqual(result.returncode, 0, result.stderr)
                payload = json.loads(result.stdout)
                self.assertTrue(payload["ok"])
                assert_schema_valid(self, schema_name, payload)

    def test_smoke_playtest_fixture_is_available_for_done_check(self) -> None:
        result = run_bootstrap("playtest", "--scenario", "smoke", "--json")
        self.assertEqual(result.returncode, 0, result.stderr)
        payload = json.loads(result.stdout)

        self.assertTrue(payload["ok"])
        self.assertEqual(payload["scenario"], "smoke")
        assertion_names = {item["name"] for item in payload["assertions"]}
        self.assertTrue(
            {
                "project_manifest_valid",
                "demo_game_contract_valid",
                "open_world_contract_valid",
                "no_crash",
            }.issubset(assertion_names)
        )
        self.assertIn("runtime smoke remains a gate item", payload["assertions"][-1]["message"])
        assert_schema_valid(self, "playtest_result.schema.json", payload)

    def test_world_generate_dry_run_plans_starter_grid_without_writing(self) -> None:
        output = REPO_ROOT / "examples/generated_world_preview"
        shutil.rmtree(output, ignore_errors=True)

        result = run_bootstrap(
            "world",
            "generate",
            "--template",
            "starter_open_world",
            "--output",
            "examples/generated_world_preview",
            "--name",
            "test_world",
            "--dry-run",
            "--json",
        )
        self.assertEqual(result.returncode, 0, result.stderr)
        payload = json.loads(result.stdout)

        self.assertFalse(output.exists())
        self.assertTrue(payload["ok"])
        self.assertTrue(payload["dry_run"])
        self.assertEqual(payload["template"], "starter_open_world")
        self.assertEqual(payload["template_version"], 1)
        self.assertEqual(payload["name"], "test_world")
        self.assertEqual(len(payload["planned_files"]), 11)
        kinds = [item["kind"] for item in payload["planned_files"]]
        self.assertEqual(kinds.count("world"), 1)
        self.assertEqual(kinds.count("sector"), 9)
        self.assertEqual(kinds.count("playtest"), 1)
        paths = {item["path"] for item in payload["planned_files"]}
        self.assertIn("examples/generated_world_preview/assets/worlds/test_world.ron", paths)
        self.assertIn("examples/generated_world_preview/assets/sectors/test_world_sector_0_0.ron", paths)
        self.assertIn("examples/generated_world_preview/assets/playtests/test_world_traversal_smoke.ron", paths)
        self.assertTrue(all(item["action"] == "create" for item in payload["planned_files"]))
        self.assertTrue(all(item["hash"].startswith("sha256:") for item in payload["planned_files"]))
        assert_schema_valid(self, "world_generate_result.schema.json", payload)

    def test_world_generate_rejects_unknown_template(self) -> None:
        result = run_bootstrap(
            "world",
            "generate",
            "--template",
            "giant_monolith_scene",
            "--output",
            "examples/generated_world_preview",
            "--name",
            "bad_world",
            "--dry-run",
            "--json",
        )
        self.assertEqual(result.returncode, 1)
        payload = json.loads(result.stdout)

        self.assertFalse(payload["ok"])
        self.assertTrue(any(item["code"] == "UNKNOWN_TEMPLATE" for item in payload["diagnostics"]))
        assert_schema_valid(self, "world_generate_result.schema.json", payload)

    def test_ability_graph_fireball_includes_granting_assets(self) -> None:
        result = run_bootstrap("ability", "graph", "fireball", "--project", "examples/demo_game_contract", "--json")
        self.assertEqual(result.returncode, 0, result.stderr)
        payload = json.loads(result.stdout)

        node_kinds = {node["kind"] for node in payload["nodes"]}
        node_ids = {node["id"] for node in payload["nodes"]}
        self.assertTrue({"ability", "effect", "tag", "registrar", "action_set", "experience"}.issubset(node_kinds))
        self.assertIn("effect:fireball_cost", node_ids)
        self.assertIn("action_set:demo_combat", node_ids)
        self.assertIn("experience:demo_combat", node_ids)
        assert_schema_valid(self, "ability_graph_result.schema.json", payload)

    def test_scene_list_and_inspect_sector_entities(self) -> None:
        list_result = run_bootstrap(
            "scene",
            "list",
            "--scene",
            "examples/open_world_studio/assets/sectors/sector_0_0.ron",
            "--json",
        )
        self.assertEqual(list_result.returncode, 0, list_result.stderr)
        list_payload = json.loads(list_result.stdout)

        self.assertTrue(list_payload["ok"])
        self.assertEqual(list_payload["kind"], "sector")
        self.assertEqual(list_payload["entity_count"], 2)
        entity_ids = {entity["id"] for entity in list_payload["entities"]}
        self.assertIn("sector_0_0/entity_0", entity_ids)
        self.assertIn("sector_0_0/entity_1", entity_ids)
        assert_schema_valid(self, "scene_list_result.schema.json", list_payload)

        inspect_result = run_bootstrap(
            "scene",
            "inspect",
            "sector_0_0/entity_0",
            "--scene",
            "examples/open_world_studio/assets/sectors/sector_0_0.ron",
            "--json",
        )
        self.assertEqual(inspect_result.returncode, 0, inspect_result.stderr)
        inspect_payload = json.loads(inspect_result.stdout)

        self.assertTrue(inspect_payload["ok"])
        self.assertEqual(inspect_payload["entity"]["prefab"], "assets/prefabs/camp_fire.ron")
        self.assertEqual(inspect_payload["entity"]["transform"]["translation"], [32.0, 0.0, -18.0])
        assert_schema_valid(self, "scene_inspect_result.schema.json", inspect_payload)

    def test_scene_inspect_reports_missing_entity(self) -> None:
        result = run_bootstrap(
            "scene",
            "inspect",
            "sector_0_0/missing",
            "--scene",
            "examples/open_world_studio/assets/sectors/sector_0_0.ron",
            "--json",
        )
        self.assertEqual(result.returncode, 1)
        payload = json.loads(result.stdout)

        self.assertFalse(payload["ok"])
        self.assertEqual(payload["diagnostics"][0]["code"], "ENTITY_NOT_FOUND")
        assert_schema_valid(self, "scene_inspect_result.schema.json", payload)

    def test_scene_patch_dry_run_reports_affected_files_without_mutation(self) -> None:
        sector = REPO_ROOT / "examples/open_world_studio/assets/sectors/sector_0_0.ron"
        before = sector.read_text(encoding="utf-8")
        result = run_bootstrap(
            "scene",
            "patch",
            "--scene",
            "examples/open_world_studio/assets/sectors/sector_0_0.ron",
            "--patch",
            "docs/specs/fixtures/open_world_studio/add_campfire.scene_patch.json",
            "--dry-run",
            "--json",
        )
        after = sector.read_text(encoding="utf-8")
        self.assertEqual(result.returncode, 0, result.stderr)
        payload = json.loads(result.stdout)

        self.assertEqual(before, after)
        self.assertTrue(payload["ok"])
        self.assertTrue(payload["dry_run"])
        self.assertEqual(payload["patch_id"], "add_campfire_preview")
        self.assertIn("assets/sectors/sector_0_0.ron", payload["affected_files"])
        self.assertIn("assets/prefabs/camp_fire.ron", payload["affected_files"])
        self.assertIn("sector_0_0/camp_fire_preview", payload["affected_entities"])
        self.assertTrue(payload["undo_token"].startswith("undo:dry-run:"))
        assert_schema_valid(self, "scene_patch_result.schema.json", payload)

    def test_scene_patch_dry_run_reports_target_mismatch(self) -> None:
        with tempfile.TemporaryDirectory() as temp:
            patch = Path(temp) / "bad_patch.json"
            patch.write_text(
                json.dumps(
                    {
                        "schema_version": 1,
                        "id": "bad_target",
                        "target": {
                            "path": "assets/sectors/sector_9_9.ron",
                            "kind": "sector",
                        },
                        "ops": [
                            {
                                "SetDataLayer": {
                                    "entity_id": "sector_0_0/camp_fire_preview",
                                    "layer": "encounters",
                                    "enabled": True,
                                }
                            }
                        ],
                    }
                ),
                encoding="utf-8",
            )
            result = run_bootstrap(
                "scene",
                "patch",
                "--scene",
                "examples/open_world_studio/assets/sectors/sector_0_0.ron",
                "--patch",
                str(patch),
                "--dry-run",
                "--json",
            )
            self.assertEqual(result.returncode, 1)
            payload = json.loads(result.stdout)

            self.assertFalse(payload["ok"])
            self.assertTrue(any(item["code"] == "TARGET_MISMATCH" for item in payload["diagnostics"]))
            assert_schema_valid(self, "scene_patch_result.schema.json", payload)

    def test_ability_graph_reports_unregistered_tags(self) -> None:
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            (root / "aa.project.toml").write_text(
                """\
schema_version = 1
name = "ability_graph_fixture"
version = "0.1.0"

[engine]
config_root = "config"
assets_root = "assets"

[plugins]
runtime = []

[features]
enabled = []

[build]
default_binary = "ability_graph_fixture"
""",
                encoding="utf-8",
            )
            ability_dir = root / "assets/abilities"
            tags_dir = root / "assets/data"
            ability_dir.mkdir(parents=True)
            tags_dir.mkdir(parents=True)
            (ability_dir / "bad_fire.ron").write_text(
                """\
(
    schema_version: 1,
    id: "bad_fire",
    cooldown_tags: ["Cooldown.BadFire"],
    activation_tags_required: ["State.CanCast"],
    activation_tags_blocked: [],
    cost_effect: null,
    impl: "aa_ability::abilities::bad_fire",
)
""",
                encoding="utf-8",
            )
            (tags_dir / "tags.ron").write_text(
                """\
(
    schema_version: 1,
    tags: ["State.Ready"],
)
""",
                encoding="utf-8",
            )

            result = run_bootstrap("ability", "graph", "bad_fire", "--project", str(root), "--json")
            self.assertEqual(result.returncode, 1)
            payload = json.loads(result.stdout)

            self.assertFalse(payload["ok"])
            self.assertEqual(payload["ability_id"], "bad_fire")
            self.assertTrue(any(item["code"] == "TAG_UNREGISTERED" for item in payload["diagnostics"]))
            assert_schema_valid(self, "ability_graph_result.schema.json", payload)

    @unittest.skipIf(shutil.which("cargo") is None, "cargo not available")
    def test_check_good_cargo_project_matches_schema(self) -> None:
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            write_cargo_project(root, "pub fn answer() -> u32 { 42 }\n")

            result = run_bootstrap("check", str(root), "--json")
            self.assertEqual(result.returncode, 0, result.stderr)
            payload = json.loads(result.stdout)

            self.assertTrue(payload["ok"])
            self.assertEqual(payload["error_count"], 0)
            assert_schema_valid(self, "check_result.schema.json", payload)

    @unittest.skipIf(shutil.which("cargo") is None, "cargo not available")
    def test_check_broken_cargo_project_reports_schema_valid_error(self) -> None:
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            write_cargo_project(root, "pub fn answer() -> u32 { \"wrong\" }\n")

            result = run_bootstrap("check", str(root), "--json")
            self.assertEqual(result.returncode, 2)
            payload = json.loads(result.stdout)

            self.assertFalse(payload["ok"])
            self.assertGreaterEqual(payload["error_count"], 1)
            self.assertTrue(any(item["severity"] == "error" for item in payload["diagnostics"]))
            assert_schema_valid(self, "check_result.schema.json", payload)


if __name__ == "__main__":
    unittest.main()
