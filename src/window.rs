use std::mem;
use xcb::x;
use xcb::Xid;
use crate::display::{self, DisplaySettings, Rgb};

fn conn_to_cairo_xcb(conn: &xcb::Connection) -> cairo::XCBConnection {
    unsafe {
        let raw = mem::transmute::<
            *mut xcb::ffi::xcb_connection_t,
            *mut cairo::ffi::xcb_connection_t,
        >(conn.get_raw_conn());

        cairo::XCBConnection::from_raw_full(raw)
    }
}

fn window_to_cairo_drawable(window: &x::Window) -> cairo::XCBDrawable {
    let id = window.resource_id();
    cairo::XCBDrawable(id)
}

fn xcb_visualtype_to_cairo(visualtype: &x::Visualtype) -> cairo::XCBVisualType {
    unsafe { mem::transmute(visualtype) }
}

pub struct Window<'a> {
    conn: &'a xcb::Connection,
    window: x::Window,
    parent_screen: &'a x::Screen,
    display: display::Display,
}

impl<'a> Window<'a> {
    pub fn new(conn: &'a xcb::Connection, screen_num: i32, width: u16, height: u16) -> anyhow::Result<Self> {
        use x::Cw;

        let window: x::Window = conn.generate_id();

        let screen = conn.get_setup().roots().nth(screen_num as usize).expect("screen dissapeared");

        let scr_w = screen.width_in_pixels();
        let scr_h = screen.height_in_pixels();

        let values = &[
            Cw::BackPixel(screen.white_pixel()),
            Cw::OverrideRedirect(true),
            Cw::EventMask(x::EventMask::EXPOSURE | x::EventMask::KEY_PRESS),
        ];

        let create_window_req = x::CreateWindow {
            depth: x::COPY_FROM_PARENT as u8,
            wid: window,
            parent: screen.root(),
            x: (scr_w / 2 - width / 2) as i16,
            y: (scr_h / 2 - height / 2) as i16,
            width,
            height,
            border_width: 0,
            class: x::WindowClass::InputOutput,
            visual: screen.root_visual(),
            value_list: values,
        };

        let cookie = conn.send_request_checked(&create_window_req);
        conn.check_request(cookie)?;

        let visualtype = screen.allowed_depths()
            .find_map(|dpts| dpts.visuals()
                .iter()
                .find(|v| v.visual_id() == screen.root_visual()))
            .expect("could not find visualtype");

        let display = display::Display::new(
            &conn_to_cairo_xcb(conn),
            &window_to_cairo_drawable(&window),
            &xcb_visualtype_to_cairo(visualtype),
            DisplaySettings {
                font_name: "monospace".to_owned(),
                font_size: 24.0,
                full_width: width as i32,
                full_height: height as i32,
                main_bg: Rgb::from_hex(0x000000),
                prompt_bg: Rgb::from_hex(0x11111b),
                prompt_fg: Rgb::from_hex(0xeeeeee),
            },
        )?;

        Ok(Self {
            conn,
            window,
            parent_screen: screen,
            display,
        })
    }

    pub fn get_intern_atom(&self, name: &[u8]) -> xcb::Result<x::Atom> {
        let intern_atom_req = x::InternAtom {
            only_if_exists: true,
            name,
        };

        let cookie = self.conn.send_request(&intern_atom_req);

        self.conn.wait_for_reply(cookie).map(|v| v.atom())
    }

    pub fn replace_atom(&self, atom: x::Atom, data: &[u8]) -> xcb::Result<()> {
        let change_prop_req = x::ChangeProperty {
            mode: x::PropMode::Replace,
            window: self.window,
            property: atom,
            r#type: x::ATOM_ATOM,
            data,
        };

        let cookie = self.conn.send_request_checked(&change_prop_req);

        self.conn.check_request(cookie)?;

        Ok(())
    }

    pub fn map_now(&self) -> xcb::Result<()> {
        self.conn.send_request(&x::MapWindow { window: self.window });
        self.conn.flush()?;
        Ok(())
    }

    pub fn next_event(&self) -> xcb::Result<xcb::Event> {
        self.conn.wait_for_event()
    }

    pub fn set_focus(&self) -> xcb::Result<()> {
        let set_focus_req = x::SetInputFocus {
            revert_to: x::InputFocus::PointerRoot,
            focus: self.window, 
            time: 0,
        };

        let cookie = self.conn.send_request_checked(&set_focus_req);
        self.conn.check_request(cookie)?;

        Ok(())
    }

    pub fn grab_keyboard(&self) -> xcb::Result<x::GrabStatus> {
        let grab_keyboard_req = x::GrabKeyboard {
            owner_events: true,
            grab_window: self.window,
            time: 0,
            pointer_mode: x::GrabMode::Async,
            keyboard_mode: x::GrabMode::Async,
        };

        let cookie = self.conn.send_request(&grab_keyboard_req);
        let reply = self.conn.wait_for_reply(cookie)?;

        Ok(reply.status())
    }

    pub fn conn(&self) -> &'a xcb::Connection {
        self.conn
    }

    pub fn display(&self) -> &display::Display {
        &self.display
    }
}
