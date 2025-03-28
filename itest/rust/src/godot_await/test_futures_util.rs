use std::time::Instant;

use futures_util::{join, pin_mut, select, FutureExt};

use godot::classes::{Button, Engine, Node, SceneTree};
use godot::meta::ToGodot;
use godot::obj::{Gd, NewAlloc};
use godot::task;
use godot::task::TaskHandle;

// use godot_await::futures::{or, zip, FutureExt};
use godot_await::prelude::*;

use crate::framework::{itest, TestContext};

fn get_tree() -> Gd<SceneTree> {
    Engine::singleton()
        .get_main_loop()
        .expect("ERR(godot_await):get MainLoop failed")
        .try_cast()
        .expect("ERR(godot_await):cast SceneTree failed")
}

#[itest(async)]
fn wait_test() -> TaskHandle {
    task::spawn(async move {
        let start = Instant::now();
        wait(0.5).await;
        println!("wait elasp sec:{}", (Instant::now() - start).as_secs_f32());
        assert!((Instant::now() - start).as_secs_f32() >= 0.4);

        let start = Instant::now();
        wait_ex(&mut get_tree(), 0.1)
            .process_always(false)
            .done()
            .await;
        println!(
            "wait_ex elasp sec:{}",
            (Instant::now() - start).as_secs_f32()
        );
        assert!((Instant::now() - start).as_secs_f32() >= 0.09);
    })
}

#[itest(async)]
fn node_test(test_context: &TestContext) -> TaskHandle {
    let node = Node::new_alloc();
    let node_ref = node.clone();
    let mut tree = test_context.scene_tree.clone();

    task::spawn(async move {
        node_ref.tree_entered().await;
    });

    tree.add_child(&node);

    let node_ref = node.clone();

    let task_handle = task::spawn(async move {
        node_ref.tree_exited().await;
    });
    tree.remove_child(&node);

    task_handle
}

// #[itest(async)]
// fn button_panic_test(test_context: &TestContext) -> TaskHandle {
//     let mut button = Button::new_alloc();
//     let button_ref = button.clone();

//     let task_handle = task::spawn(async move {
//         button_ref.pressed().await;
//     });

//     // button.emit_signal("pressed", &[]);
//     button.call_deferred("free", &[]);

//     task_handle
// }

#[itest(async)]
fn button_test() -> TaskHandle {
    let mut button = Button::new_alloc();
    let button_ref = button.clone();

    let task_handle = task::spawn(async move {
        let fut_1 = async {
            join!(
                button_ref.button_down(),
                button_ref.button_up(),
                button_ref.pressed(),
                button_ref.toggled(),
            )
        }
        .fuse();

        let fut_2 = wait(0.5).fuse();
        pin_mut!(fut_1); // Pins the Future on the stack
        pin_mut!(fut_2); // Pins the Future on the stack

        select! {
            ret= fut_1 =>{
                assert_eq!(ret, ((), (), (), (true,)))
            }
                ,
            _ = fut_2 => panic!("test wait exceed"),
        };
    });

    button.emit_signal("button_down", &[]);
    button.emit_signal("button_up", &[]);
    button.emit_signal("toggled", &[true.to_variant()]);
    button.emit_signal("pressed", &[]);

    task_handle
}
