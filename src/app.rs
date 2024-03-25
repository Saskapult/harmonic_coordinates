/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct HarmonicApp {
	// Example stuff:
	label: String,

	#[serde(skip)] // This how you opt-out of serialization of a field
	value: f32,
}

impl Default for HarmonicApp {
	fn default() -> Self {
		Self {
			// Example stuff:
			label: "Hello World!".to_owned(),
			value: 2.7,
		}
	}
}
// butt
impl HarmonicApp {
	/// Called once before the first frame.
	pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
		// This is also where you can customize the look and feel of egui using
		// `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.

		let wgpu = cc.wgpu_render_state.as_ref()
			.expect("Wgpu state not found!");
		// Init rendering thing, store in arc mutex

		// Load previous app state (if any).
		// Note that you must enable the `persistence` feature for this to work.
		if let Some(storage) = cc.storage {
			return eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
		}

		Default::default()
	}

	fn custom_painting(&mut self, ui: &mut egui::Ui) {
		let (rect, response) = ui.allocate_exact_size(egui::Vec2::splat(300.0), egui::Sense::drag());
		// self.angle += response.drag_motion().x * 0.01;
		let callback = egui::PaintCallback {
			rect,
			callback: std::sync::Arc::new(eframe::wgpu::CallbackFn::new(move |_info, painter| {
				rotating_triangle.lock().paint(painter.gl(), angle);
			})),
		};
		ui.painter().add(callback);
	}
}

impl eframe::App for HarmonicApp {
	/// Called by the frame work to save state before shutdown.
	fn save(&mut self, storage: &mut dyn eframe::Storage) {
		eframe::set_value(storage, eframe::APP_KEY, self);
	}

	/// Called each time the UI needs repainting, which may be many times per second.
	fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
		egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
			egui::menu::bar(ui, |ui| {
				// NOTE: no File->Quit on web pages!
				let is_web = cfg!(target_arch = "wasm32");
				if !is_web {
					ui.menu_button("File", |ui| {
						if ui.button("Quit").clicked() {
							ctx.send_viewport_cmd(egui::ViewportCommand::Close);
						}
					});
					ui.add_space(16.0);
				}

				if ui.button("Load Cage").clicked() {
					// ctx.send_viewport_cmd(egui::ViewportCommand::Close);
				}
				if ui.button("Recompute Grid").clicked() {
					// ctx.send_viewport_cmd(egui::ViewportCommand::Close);
				}
				if ui.button("Load Mesh").clicked() {
					// ctx.send_viewport_cmd(egui::ViewportCommand::Close);
				}
				if ui.button("Weight Mesh").clicked() {
					// ctx.send_viewport_cmd(egui::ViewportCommand::Close);
				}
				
				// egui::widgets::global_dark_light_mode_buttons(ui);
			});
		});

		// egui::Window::new("File Load context")

		egui::CentralPanel::default().show(ctx, |ui| {
			// The central panel the region left after adding TopPanel's and SidePanel's
			ui.heading("eframe template");

			ui.horizontal(|ui| {
				ui.label("Write something: ");
				ui.text_edit_singleline(&mut self.label);
			});

			ui.add(egui::Slider::new(&mut self.value, 0.0..=10.0).text("value"));
			if ui.button("Increment").clicked() {
				self.value += 1.0;
			}

			ui.separator();

			ui.add(egui::github_link_file!(
				"https://github.com/emilk/eframe_template/blob/master/",
				"Source code."
			));

			ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
				powered_by_egui_and_eframe(ui);
				egui::warn_if_debug_build(ui);
			});
		});
	}
}

fn powered_by_egui_and_eframe(ui: &mut egui::Ui) {
	ui.horizontal(|ui| {
		ui.spacing_mut().item_spacing.x = 0.0;
		ui.label("Powered by ");
		ui.hyperlink_to("egui", "https://github.com/emilk/egui");
		ui.label(" and ");
		ui.hyperlink_to(
			"eframe",
			"https://github.com/emilk/egui/tree/master/crates/eframe",
		);
		ui.label(".");
	});
}