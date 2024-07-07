use winit::{
    event::{self, Event, RawKeyEvent, WindowEvent},
    event_loop::EventLoop,
    keyboard::KeyCode,
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

pub struct Application<T: ApplicationTrait> {
    application: T,
}

impl<T: ApplicationTrait> Application<T> {
    pub fn new(event_loop: &EventLoop<()>) -> Application<T> {
        Self {
            application: T::on_new(event_loop),
        }
    }

    pub fn run(&mut self, event_loop: EventLoop<()>) {
        event_loop
            .run(move |event, _control_flow| {
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
                    Event::DeviceEvent {
                        device_id,
                        ref event,
                    } => match event {
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
            })
            .unwrap();
    }
}
