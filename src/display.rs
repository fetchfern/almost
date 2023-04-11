use std::cell::RefCell;
use anyhow::Context as AnyhowContext;
use cairo::{
    Context,
    XCBSurface,
    XCBConnection,
    XCBDrawable,
    XCBVisualType,
    FontFace,
    FontSlant,
    FontWeight,
};

#[derive(Default, Debug)]
pub struct LauncherState {
    pub prompt: String,
}

#[derive(Debug)]
pub struct Rgb {
    r: f64,
    g: f64,
    b: f64,
}

impl Rgb {
    pub fn new(r: f64, g: f64, b: f64) -> Self {
        Rgb { r, g, b }
    }

    pub fn from_hex(hex: u32) -> Self {
        let r = (hex >> 16) & 0xFF;
        let g = (hex >> 8) & 0xFF;
        let b = hex & 0xFF;

        Rgb::new(r as f64 / 255., g as f64 / 255., b as f64 / 255.)
    }

    pub fn components(&self) -> (f64, f64, f64) {
        (self.r, self.g, self.b)
    }
}

#[derive(Debug)]
pub struct DisplaySettings {
    pub font_name: String,
    pub font_size: f64,
    pub full_width: i32,
    pub full_height: i32,
    pub main_bg: Rgb,
    pub prompt_bg: Rgb,
    pub prompt_fg: Rgb,
}

pub struct Display {
    context: Context,
    surface: XCBSurface,
    state: RefCell<LauncherState>,
    settings: DisplaySettings,
    cached_font: FontFace,
    font_real_height: f64,
}

impl Display {
    pub fn new(
        conn: &XCBConnection,
        drawable: &XCBDrawable,
        visualtype: &XCBVisualType,
        settings: DisplaySettings,
    ) -> anyhow::Result<Self> {
        let surface = XCBSurface::create(conn, drawable, visualtype, settings.full_width, settings.full_height)
            .context("failed to create cairo surface")?;

        let context = Context::new(&surface)
            .context("failed to create cairo context")?;

        let font = FontFace::toy_create(&settings.font_name, FontSlant::Normal, FontWeight::Normal)
            .context(format!("failed to create font face for '{}'", settings.font_name))?;

        context.set_font_face(&font);
        let extents = context.font_extents()?;
        let font_real_height = extents.height();

        Ok(Self {
            surface,
            context,
            settings,
            state: RefCell::new(LauncherState::default()),
            cached_font: font,
            font_real_height,
        })
    }

    pub fn redraw(&self) -> anyhow::Result<()> {
        let ctx = &self.context;
        let settings = &self.settings;
        let state = &self.state;

        println!("{settings:#?}");

        const FONT_PAD_WITHIN_PROMPT: f64 = 28.;

        ctx.move_to(0., 0.);

        // draw prompt bg
        let (r, g, b) = settings.prompt_bg.components();
        ctx.set_source_rgb(r, g, b);
        ctx.rectangle(0., 0., settings.full_width as f64, self.font_real_height + FONT_PAD_WITHIN_PROMPT);
        ctx.fill()?;
        
        // draw prompt text
        let (r, g, b) = settings.prompt_fg.components();
        ctx.set_source_rgb(r, g, b);
        ctx.set_font_face(&self.cached_font);
        ctx.set_font_size(settings.font_size);
        ctx.move_to(4., self.font_real_height + FONT_PAD_WITHIN_PROMPT / 2.);
        ctx.show_text(&state.borrow().prompt)?;

        self.surface.flush();

        Ok(())
    }

    pub fn update_state<F>(&self, predicate: F)
    where
        F: Fn(&mut LauncherState),
    {
        let state = &mut self.state.borrow_mut();
        predicate(state);
    }
}
