//! Visual Evaluation Loop (VEL) application for PhysLang v0.5
//!
//! This module provides an interactive visualization of PhysLang programs
//! with live editing support via file watching.

use eframe::egui;
use notify::{Event, RecommendedWatcher, Watcher};
use physlang_core::{
    build_simulation_context_from_source, get_particle_states, step_simulation,
    SimulationContext,
};
use std::path::PathBuf;
use std::sync::mpsc;

/// Visual Evaluation Loop application
pub struct VelApp {
    source_path: PathBuf,
    source_text: String,
    ctx_opt: Option<SimulationContext>,
    last_load_error: Option<String>,
    playing: bool,
    speed_multiplier: f32,
    #[allow(dead_code)] // Kept alive to maintain file watching
    file_watcher: Option<RecommendedWatcher>,
    file_receiver: mpsc::Receiver<notify::Result<Event>>,
    needs_reload: bool,
}

impl VelApp {
    pub fn new(source_path: PathBuf, _cc: &eframe::CreationContext<'_>) -> Self {
        let source_text = std::fs::read_to_string(&source_path)
            .unwrap_or_else(|e| format!("Error reading file: {}", e));

        let (tx, rx) = mpsc::channel();
        let mut watcher = notify::recommended_watcher(move |res| {
            // Silently ignore broken pipe errors - they can happen during shutdown
            let _ = tx.send(res);
        }).ok();

        // Start watching the file
        if let Some(ref mut w) = watcher {
            if let Err(e) = w.watch(&source_path, notify::RecursiveMode::NonRecursive) {
                // Don't print to stderr here to avoid broken pipe issues
                // File watching is optional for VEL functionality
            }
        }

        let mut app = Self {
            source_path,
            source_text: source_text.clone(),
            ctx_opt: None,
            last_load_error: None,
            playing: false,
            speed_multiplier: 1.0,
            file_watcher: watcher,
            file_receiver: rx,
            needs_reload: false,
        };

        // Initial load
        app.reload_context();

        app
    }

    fn reload_context(&mut self) {
        match build_simulation_context_from_source(&self.source_text) {
            Ok((mut ctx, _diagnostics)) => {
                ctx.current_step = 0;
                self.ctx_opt = Some(ctx);
                self.last_load_error = None;
            }
            Err(e) => {
                self.last_load_error = Some(format!("{}", e));
                self.ctx_opt = None;
                self.playing = false;
            }
        }
    }

    fn check_file_changes(&mut self) {
        // Check for file change events
        while let Ok(event) = self.file_receiver.try_recv() {
            match event {
                Ok(Event {
                    kind: notify::EventKind::Modify(_),
                    paths,
                    ..
                }) => {
                    if paths.contains(&self.source_path) {
                        // Re-read the file
                        if let Ok(new_text) = std::fs::read_to_string(&self.source_path) {
                            self.source_text = new_text;
                            self.needs_reload = true;
                        }
                    }
                }
                Ok(_) => {}
                Err(e) => {
                    eprintln!("File watcher error: {}", e);
                }
            }
        }

        if self.needs_reload {
            self.reload_context();
            self.needs_reload = false;
        }
    }
}

impl eframe::App for VelApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Check for file changes
        self.check_file_changes();

        // Top bar with controls
        egui::TopBottomPanel::top("controls").show(ctx, |ui| {
            ui.horizontal(|ui| {
                // Play/Pause button
                if ui.button(if self.playing { "⏸ Pause" } else { "▶ Play" }).clicked() {
                    self.playing = !self.playing;
                }

                // Reset button
                if ui.button("⏮ Reset").clicked() {
                    self.reload_context();
                    self.playing = false;
                }

                // Step button
                if ui.button("⏭ Step").clicked() {
                    if let Some(ref mut ctx) = self.ctx_opt {
                        step_simulation(ctx);
                    }
                }

                ui.separator();

                // Speed control
                ui.label("Speed:");
                ui.add(egui::Slider::new(&mut self.speed_multiplier, 0.1..=10.0));

                ui.separator();

                // Step counter
                if let Some(ref ctx) = self.ctx_opt {
                    ui.label(format!(
                        "Step: {} / {}",
                        ctx.current_step, ctx.max_steps
                    ));
                }
            });
        });

        // Main canvas area
        egui::CentralPanel::default().show(ctx, |ui| {
            // Draw particles and forces
            let rect = ui.max_rect();
            let painter = ui.painter();

            // Coordinate transformation: map world coordinates to screen
            // Assume world coordinates are roughly in [-20, 20] range
            let world_range = 40.0; // from -20 to 20
            let center = rect.center();
            let scale = (rect.width().min(rect.height()) / world_range) * 0.9;

            // Draw springs (forces)
            if let Some(ref ctx) = self.ctx_opt {
                for force in &ctx.world.forces {
                    match force {
                        physlang_core::Force::Spring { a, b, .. } => {
                            let pos_a = ctx.world.particles[*a].pos;
                            let pos_b = ctx.world.particles[*b].pos;

                            let screen_a = center
                                + egui::vec2(pos_a.x * scale, -pos_a.y * scale);
                            let screen_b = center
                                + egui::vec2(pos_b.x * scale, -pos_b.y * scale);

                            painter.line_segment(
                                [screen_a, screen_b],
                                egui::Stroke::new(1.0, egui::Color32::GRAY),
                            );
                        }
                        _ => {} // Only draw springs for now
                    }
                }

                // Draw particles
                let particle_states = get_particle_states(ctx);
                for particle in particle_states {
                    let screen_pos = center
                        + egui::vec2(particle.pos.x * scale, -particle.pos.y * scale);

                    // Particle radius based on mass (with reasonable bounds)
                    let radius = (particle.mass.sqrt() * scale * 0.5).max(3.0).min(20.0);

                    painter.circle_filled(screen_pos, radius, egui::Color32::LIGHT_BLUE);
                    painter.circle_stroke(screen_pos, radius, egui::Stroke::new(1.0, egui::Color32::BLUE));

                    // Draw particle name
                    painter.text(
                        screen_pos + egui::vec2(0.0, radius + 10.0),
                        egui::Align2::CENTER_TOP,
                        &particle.name,
                        egui::FontId::default(),
                        egui::Color32::WHITE,
                    );
                }
            }

            // Show error message if any
            if let Some(ref error) = self.last_load_error {
                ui.vertical_centered(|ui| {
                    ui.add_space(rect.height() * 0.4);
                    ui.label(
                        egui::RichText::new(format!("Error: {}", error))
                            .color(egui::Color32::RED)
                            .size(16.0),
                    );
                });
            }
        });

        // Bottom panel for errors/diagnostics
        if self.last_load_error.is_some() {
            egui::TopBottomPanel::bottom("errors").show(ctx, |ui| {
                ui.set_max_height(100.0);
                if let Some(ref error) = self.last_load_error {
                    ui.label(
                        egui::RichText::new(format!("Error: {}", error))
                            .color(egui::Color32::RED),
                    );
                }
            });
        }

        // Simulation stepping
        if self.playing {
            if let Some(ref mut ctx) = self.ctx_opt {
                let steps_per_frame = self.speed_multiplier.max(0.1).round() as usize;
                for _ in 0..steps_per_frame {
                    if step_simulation(ctx) {
                        // Simulation finished
                        self.playing = false;
                        break;
                    }
                }
            }
        }

        // Request repaint for animation
        if self.playing {
            ctx.request_repaint();
        }
    }
}

