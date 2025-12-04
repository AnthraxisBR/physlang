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
            if let Err(_e) = w.watch(&source_path, notify::RecursiveMode::NonRecursive) {
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

            // Calculate dynamic viewport based on actual particle positions
            let (center, scale, world_center_x, world_center_y) = if let Some(ref ctx) = self.ctx_opt {
                let particle_states = get_particle_states(ctx);
                
                if particle_states.is_empty() {
                    // No particles: use default view
                    (rect.center(), 1.0, 0.0, 0.0)
                } else {
                    // Calculate bounding box of all particles
                    let mut min_x = f32::INFINITY;
                    let mut max_x = f32::NEG_INFINITY;
                    let mut min_y = f32::INFINITY;
                    let mut max_y = f32::NEG_INFINITY;
                    
                    for particle in &particle_states {
                        min_x = min_x.min(particle.pos.x);
                        max_x = max_x.max(particle.pos.x);
                        min_y = min_y.min(particle.pos.y);
                        max_y = max_y.max(particle.pos.y);
                    }
                    
                    // Add padding (10% on each side, minimum 2.0 units)
                    let padding = ((max_x - min_x).max(max_y - min_y) * 0.1).max(2.0);
                    let world_width = (max_x - min_x + 2.0 * padding).max(1.0);
                    let world_height = (max_y - min_y + 2.0 * padding).max(1.0);
                    
                    // Calculate center of bounding box
                    let world_center_x = (min_x + max_x) / 2.0;
                    let world_center_y = (min_y + max_y) / 2.0;
                    
                    // Calculate scale to fit bounding box in viewport
                    let scale_x = (rect.width() * 0.9) / world_width;
                    let scale_y = (rect.height() * 0.9) / world_height;
                    let scale = scale_x.min(scale_y);
                    
                    // Screen center
                    let screen_center = rect.center();
                    
                    (screen_center, scale, world_center_x, world_center_y)
                }
            } else {
                // No context: use default view
                (rect.center(), 1.0, 0.0, 0.0)
            };

            // Draw springs (forces)
            if let Some(ref ctx) = self.ctx_opt {
                for force in &ctx.world.forces {
                    match force {
                        physlang_core::Force::Spring { a, b, .. } => {
                            let pos_a = ctx.world.particles[*a].pos;
                            let pos_b = ctx.world.particles[*b].pos;

                            // Transform world coordinates to screen coordinates
                            let screen_a = center
                                + egui::vec2(
                                    (pos_a.x - world_center_x) * scale,
                                    -(pos_a.y - world_center_y) * scale,
                                );
                            let screen_b = center
                                + egui::vec2(
                                    (pos_b.x - world_center_x) * scale,
                                    -(pos_b.y - world_center_y) * scale,
                                );

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
                    // Transform world coordinates to screen coordinates
                    let screen_pos = center
                        + egui::vec2(
                            (particle.pos.x - world_center_x) * scale,
                            -(particle.pos.y - world_center_y) * scale,
                        );

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

