use crate::{
    actions::*, asset::BehaviorDocument, complete_behavior, composites::*, decorators::*,
    start_behavior, BehaviorCursor, BehaviorInfo, BehaviorSpawner, BehaviorTrace, BehaviorTree,
};
use bevy::{
    ecs::system::{CommandQueue, EntityCommands},
    prelude::*,
    reflect::TypeUuid,
};
use serde::{Deserialize, Serialize};

pub const MAX_ITERS: usize = 200;

pub fn test_app(app: &mut App) -> &mut App {
    // Add the behaviors system to the app
    app.add_system(start_behavior);
    app.add_system(complete_behavior);
    app.add_system(sequencer::run);
    app.add_system(selector::run);
    app.add_system(all::run);
    app.add_system(any::run);
    app.add_system(repeater::run);
    app.add_system(inverter::run);
    app.add_system(succeeder::run);
    app.add_system(delay::run);
    app.add_system(debug::run);
    app.init_resource::<BehaviorTrace>();
    app
}

#[derive(Serialize, Deserialize, TypeUuid)]
#[uuid = "3d6cc56a-542e-11ed-9abb-02a179e5df2b"]
pub enum TestBehavior {
    Debug(Debug),
    Selector(Selector),
    Sequencer(Sequencer),
    All(All),
    Any(Any),
    Repeater(Repeater),
    Inverter(Inverter),
    Succeeder(Succeeder),
    Delay(Delay),
}

impl Default for TestBehavior {
    fn default() -> Self {
        Self::Debug(Debug::default())
    }
}

impl BehaviorSpawner for TestBehavior {
    fn insert(&self, commands: &mut EntityCommands) {
        match self {
            TestBehavior::Debug(data) => BehaviorInfo::insert_with(commands, data),
            TestBehavior::Selector(data) => BehaviorInfo::insert_with(commands, data),
            TestBehavior::Sequencer(data) => BehaviorInfo::insert_with(commands, data),
            TestBehavior::All(data) => BehaviorInfo::insert_with(commands, data),
            TestBehavior::Any(data) => BehaviorInfo::insert_with(commands, data),
            TestBehavior::Repeater(data) => BehaviorInfo::insert_with(commands, data),
            TestBehavior::Inverter(data) => BehaviorInfo::insert_with(commands, data),
            TestBehavior::Succeeder(data) => BehaviorInfo::insert_with(commands, data),
            TestBehavior::Delay(data) => BehaviorInfo::insert_with(commands, data),
        }
    }
}

pub fn trace_behavior(behavior: &str) -> BehaviorTrace {
    // Load behavior tree from RON string
    let document = ron::from_str::<BehaviorDocument<TestBehavior>>(behavior);
    assert!(document.is_ok());
    let document = document.unwrap();
    // println!("Loaded behavior tree: \n{:#?}", ron::ser::to_string_pretty(&document, Default::default()).unwrap());

    // Create app
    let mut app = App::new();
    app.add_plugin(bevy::time::TimePlugin::default());
    test_app(&mut app);
    let mut command_queue = CommandQueue::default();
    let mut commands = Commands::new(&mut command_queue, &app.world);

    // Spawn tree
    let entity = BehaviorTree::spawn_tree(None, &mut commands, &document.root);
    commands.entity(entity).insert(BehaviorCursor);

    // Apply commands
    command_queue.apply(&mut app.world);

    // Run app
    let mut iters = 0;
    while iters < MAX_ITERS {
        iters = iters + 1;
        app.update();
    }

    // Get app trace
    app.world.get_resource::<BehaviorTrace>().unwrap().clone()
}
