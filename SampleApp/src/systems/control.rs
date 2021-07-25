use crate::transform::Transform;
use crate::controller::Controller;
use crate::input::Input;
use crate::ecs::EntityComponentSystem;
use sdl2::keyboard::Scancode;

pub fn control_system(input: &Input, delta_time: f32, ecs: &EntityComponentSystem) {
    let mut transforms = ecs.get_component_set::<Transform>().unwrap().borrow_mut();
    let mut controllers = ecs.get_component_set::<Controller>().unwrap().borrow_mut();
    
    for entity in ecs.active_entities() {
        match (transforms.get_mut(&entity), controllers.get_mut(&entity)) {
            (Some(transform), Some(controller)) => {
                let mut acc_dir = transform.rotation*cgmath::Vector3{x: 1.0, y: 0.0, z: 0.0};
                if input.is_key_pressed(Scancode::W) {
                    acc_dir *= 1.0;
                }
                else if input.is_key_pressed(Scancode::S) {
                    acc_dir *= -1.0;
                }
                else {
                    acc_dir *= 0.0;
                }

                cgmath::Deg(1.0);

                let mut rotate_dir = 0.0;
                if input.is_key_pressed(Scancode::A) {
                    rotate_dir = 1.0;
                }
                else if input.is_key_pressed(Scancode::D) {
                    rotate_dir = -1.0;
                }

                controller.velocity = controller.acceleration_speed*delta_time*acc_dir + controller.velocity*0.99;
                transform.position += controller.velocity*delta_time;
                transform.rotation = transform.rotation*cgmath::Quaternion::from(
                    cgmath::Euler {
                        x: cgmath::Deg(0.0), 
                        y: cgmath::Deg(0.0), 
                        z: cgmath::Deg(controller.rotation_speed*rotate_dir*delta_time),
                    });
                    }
            _ => {}
        }
    }
}