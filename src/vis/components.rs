use bevy::prelude::*;


#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum AgentKind {
    Fsm,
    AStar,
    BehaviorTree,
}

#[derive(Component)]
pub struct AgentMarker {
    pub kind: AgentKind,
}

#[derive(Component)]
pub struct TrailDot;

#[derive(Component)]
pub struct GoalMarker;

#[derive(Component)]
pub struct Obstacle;

#[derive(Component)]
pub struct OrbitCamera {
    pub focus: Vec3,
    pub radius: f32,
    pub yaw: f32,   // radians
    pub pitch: f32, // radians
}

#[derive(Component)]
pub struct Shaking {
    pub timer: Timer,
    pub magnitude: f32,
}
