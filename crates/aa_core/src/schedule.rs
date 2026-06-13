use bevy::prelude::*;

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub enum AaSchedule {
    FrameStart,
    Input,
    NetReceive,
    AbilityInput,
    MovementIntent,
    AbilityFixed,
    Physics,
    Effects,
    Crowd,
    RootMotion,
    InitState,
    Animation,
    GameplayCues,
    WorldStream,
    Camera,
    WorldStreamApply,
    NetSend,
    Interpolation,
    FrameEnd,
}
