#![allow(dead_code)]
//! almost: an extendable X program launcher written in Rust.

use window::Window;
use xcb::x;

#[derive(PartialEq)]
enum Status {
    KeepGoing,
    Quit,
}

fn main() -> anyhow::Result<()> {
    let (conn, screen_num) = xcb::Connection::connect(None)?;

    let window = Window::new(&conn, screen_num, 600, 200)?;

    let net_wm_window_type = window.get_intern_atom(b"_NET_WM_WINDOW_TYPE")?;
    window.replace_atom(net_wm_window_type, b"_NET_WM_WINDOW_TYPE_DOCK")?;

    let net_wm_state = window.get_intern_atom(b"_NET_WM_STATE")?;
    window.replace_atom(net_wm_state, b"_NET_WM_STATE_DEMANDS_ATTENTION")?;
 
    window.map_now()?;

    use xcb::Event::X;
    use x::Event;

    window.display().update_state(|state| {
        state.prompt.push_str("hello, world!");
    });

    window.display().redraw()?;
    window.conn().flush()?;

    #[allow(irrefutable_let_patterns)]
    while let status = match window.next_event()? {
        X(Event::Expose(_)) => {
            println!("exposing");

            if window.set_focus().is_err() {
                println!("[warn] could not set focus to window");
            }

            window.display().redraw()?;
            window.conn().flush()?;

            Status::KeepGoing
        }

        X(Event::KeyPress(ev)) => {
            if ev.detail() == 0x18 { // Q on qwerty
                Status::Quit
            } else {
                Status::KeepGoing
            }
        },

        _ => Status::KeepGoing,
    } {
        if status == Status::Quit {
            break;
        }
    };

    Ok(())
}

pub mod display;
pub mod window;
