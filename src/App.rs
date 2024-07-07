use std::time::Duration;

use winit::{
    event::{self, Event, RawKeyEvent, WindowEvent},
    event_loop::{EventLoop, EventLoopWindowTarget},
    keyboard::KeyCode,
    platform::{pump_events::EventLoopExtPumpEvents, run_on_demand::EventLoopExtRunOnDemand},
};
pub trait ApplicationTrait {
    fn on_new(event_loop: &EventLoop<()>) -> Self;

    fn on_new_frame(&mut self, event: &Event<()>);

    fn on_draw(&mut self);

    fn on_mouse_motion(&mut self, delta: &(f64, f64));

    fn on_key_press(&mut self, key_event: &RawKeyEvent, keycode: KeyCode);

    fn on_destroy(&mut self);

    fn resize_event(&mut self);
}

pub struct App<T: ApplicationTrait> {
    application: T,
    exit: bool,
    non_block: bool,
}

impl<T: ApplicationTrait> App<T> {
    pub fn new(event_loop: &EventLoop<()>) -> App<T> {
        Self { application: T::on_new(event_loop), exit: false, non_block: false }
    }

    pub fn run(&mut self, event_loop: EventLoop<()>) {
        event_loop
            .run(move |event, _control_flow| {
                self.run_event(event, _control_flow);
            })
            .unwrap();
    }

    pub fn run_non_block(&mut self, event_loop: &mut EventLoop<()>) {
        if !self.exit {
            self.non_block = true;

            event_loop.pump_events(Some(Duration::ZERO), |event, _control_flow| {
                self.run_event(event, _control_flow);
            });

            if self.exit {
                self.application.on_destroy();
                event_loop
                    .run_on_demand(move |_, control_flow| {
                        control_flow.exit();
                    })
                    .unwrap();
            }
        }
    }

    fn run_event(&mut self, event: Event<()>, _control_flow: &EventLoopWindowTarget<()>) {
        self.application.on_new_frame(&event);

        match event {
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::CloseRequested => {
                    _control_flow.exit();
                }
                WindowEvent::Resized(extent) => {
                    self.application.resize_event();
                }
                WindowEvent::RedrawRequested => {
                    self.application.on_draw();
                }

                _ => {}
            },
            Event::DeviceEvent { device_id, ref event } => match event {
                event::DeviceEvent::MouseMotion { delta } => {
                    self.application.on_mouse_motion(delta);
                }
                event::DeviceEvent::Key(x) => match x.physical_key {
                    winit::keyboard::PhysicalKey::Code(keycode) => {
                        self.application.on_key_press(x, keycode);
                    }
                    winit::keyboard::PhysicalKey::Unidentified(_) => todo!(),
                },
                _ => {}
            },
            Event::AboutToWait => {}
            // happens after ever new event
            Event::NewEvents(_) => {}
            Event::LoopExiting => {
                // Cleanup resources here
                self.application.on_destroy();
            }

            _ => {}
        }
    }
}
