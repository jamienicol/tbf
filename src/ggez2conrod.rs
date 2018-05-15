use conrod;
use conrod::Scalar;
use ggez::Context;
use ggez;

fn convert_mouse_coords(_ctx: &Context, x: i32, y: i32) -> (Scalar, Scalar) {
    (
        // FIXME: get the actual window size from somewhere
        x as Scalar - 1280.0 / 2.0,
        (y as Scalar - 800.0 / 2.0) * -1.0,
    )
}

fn convert_mouse_button(
    _ctx: &Context,
    button: ggez::event::MouseButton,
) -> conrod::input::state::mouse::Button {
    match button {
        ggez::event::MouseButton::Unknown => conrod::input::state::mouse::Button::Unknown,
        ggez::event::MouseButton::Left => conrod::input::state::mouse::Button::Left,
        ggez::event::MouseButton::Middle => conrod::input::state::mouse::Button::Middle,
        ggez::event::MouseButton::Right => conrod::input::state::mouse::Button::Right,
        ggez::event::MouseButton::X1 => conrod::input::state::mouse::Button::X1,
        ggez::event::MouseButton::X2 => conrod::input::state::mouse::Button::X2,
    }
}

pub fn convert_mouse_motion_event(
    ctx: &Context,
    _state: ggez::event::MouseState,
    x: i32,
    y: i32,
    _xrel: i32,
    _yrel: i32,
) -> Option<conrod::event::Input> {
    let coords = convert_mouse_coords(ctx, x, y);
    let motion = conrod::input::Motion::MouseCursor {
        x: coords.0,
        y: coords.1,
    };
    Some(conrod::event::Input::Motion(motion).into())
}

pub fn convert_mouse_button_down_event(
    ctx: &mut Context,
    button: ggez::event::MouseButton,
    _x: i32,
    _y: i32,
) -> Option<conrod::event::Input> {
    let button = convert_mouse_button(ctx, button);
    Some(conrod::event::Input::Press(
        conrod::input::Button::Mouse(button).into(),
    ))
}

pub fn convert_mouse_button_up_event(
    ctx: &mut Context,
    button: ggez::event::MouseButton,
    _x: i32,
    _y: i32,
) -> Option<conrod::event::Input> {
    let button = convert_mouse_button(ctx, button);
    Some(conrod::event::Input::Release(
        conrod::input::Button::Mouse(button).into(),
    ))
}
