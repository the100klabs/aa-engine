/// Contractual exit codes from `docs/research/unreal_to_bevy/17_agent_cli_contract.md`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExitCode {
    Success = 0,
    ValidationFailed = 1,
    CompileFailed = 2,
    InvalidArgs = 4,
    InternalError = 5,
}

impl ExitCode {
    pub fn as_i32(self) -> i32 {
        self as i32
    }
}
