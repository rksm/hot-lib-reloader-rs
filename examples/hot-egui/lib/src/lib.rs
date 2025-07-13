use eframe::egui;

pub struct State {
    name: String,
    age: u32,
}

impl Default for State {
    fn default() -> Self {
        Self {
            name: "Robert".to_owned(),
            age: 36,
        }
    }
}

#[unsafe(no_mangle)]
pub fn render(state: &mut State, ctx: &egui::Context, _frame: &mut eframe::Frame) {
    egui::CentralPanel::default().show(ctx, |ui| {
        // ctx.set_pixels_per_point(2.0);
        ui.heading("My egui Application");
        ui.horizontal(|ui| {
            ui.label("Your name: ");
            ui.text_edit_singleline(&mut state.name);
        });
        ui.add(egui::Slider::new(&mut state.age, 0..=120).text("age"));
        if ui.button("Click each year").clicked() {
            state.age += 1;
        }
        ui.label(format!("Hello '{}', age {}", state.name, state.age));
    });
}
