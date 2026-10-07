//! Rendu : dessine la liste de `Sprite` avec wgpu, en WebGPU ou, à défaut, en WebGL 2.
//! Le même code servira plus tard au navigateur natif (ADR-010).

use crate::navigation::Sprite;
use wasm_bindgen::{JsCast, JsValue};
use web_sys::HtmlCanvasElement;

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct Instance {
    position: [f32; 2],
    radius: f32,
    color: [f32; 4],
}

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct Uniforms {
    aspect: f32,
    time: f32,
    reserve: [f32; 2],
}

/// Assez pour une mosaïque : un écran de 4 K découpé en points de 6 pixels.
const INSTANCES_MAX: u64 = 262_144;

pub struct Renderer {
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    pipeline: wgpu::RenderPipeline,
    instances: wgpu::Buffer,
    uniforms: wgpu::Buffer,
    bind_group: wgpu::BindGroup,
    pub backend: &'static str,
}

impl Renderer {
    /// Essaie WebGPU, puis WebGL 2. Rend aussi la zone de dessin réellement utilisée : une
    /// zone qui a reçu un contexte WebGPU ne peut plus en recevoir un autre, le repli en
    /// crée donc une neuve à la même place.
    pub async fn new_one(canvas: HtmlCanvasElement) -> Result<(Renderer, HtmlCanvasElement), JsValue> {
        let mut canvas = canvas;
        let mut last_error = String::from("aucune carte graphique accessible");
        // Pour mesurer le mode de secours sur un appareil qui a WebGPU : la page pose
        // `window.__holoSansWebGPU` (adresse en « ?webgl »), et l'on passe directement à WebGL 2.
        let without_webgpu = web_sys::window()
            .and_then(|window| js_sys::Reflect::get(&window, &JsValue::from_str("__holoSansWebGPU")).ok())
            .is_some_and(|value| value.is_truthy());
        for (backends, name) in [(wgpu::Backends::BROWSER_WEBGPU, "WebGPU"), (wgpu::Backends::GL, "WebGL 2")] {
            if without_webgpu && name == "WebGPU" {
                continue;
            }
            match Renderer::with(backends, canvas.clone()).await {
                Ok(renderer) => return Ok((renderer, canvas)),
                Err(e) => {
                    web_sys::console::warn_1(&JsValue::from_str(&format!("{name} indisponible : {e}")));
                    last_error = format!("{name} : {e}");
                    let fresh: HtmlCanvasElement = canvas.clone_node()?.dyn_into()?;
                    canvas.replace_with_with_node_1(&fresh)?;
                    canvas = fresh;
                }
            }
        }
        Err(JsValue::from_str(&format!("ni WebGPU ni WebGL 2 ne sont utilisables ({last_error})")))
    }

    async fn with(backends: wgpu::Backends, canvas: HtmlCanvasElement) -> Result<Renderer, String> {
        let width = canvas.width().max(1);
        let height = canvas.height().max(1);

        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor { backends, ..Default::default() });
        let surface = instance
            .create_surface(wgpu::SurfaceTarget::Canvas(canvas))
            .map_err(|e| format!("surface : {e}"))?;
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::LowPower,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .ok_or_else(|| String::from("aucun adaptateur"))?;
        let backend = match adapter.get_info().backend {
            wgpu::Backend::BrowserWebGpu => "WebGPU",
            wgpu::Backend::Gl => "WebGL 2",
            _ => "autre",
        };
        let limits = if backend == "WebGL 2" {
            wgpu::Limits::downlevel_webgl2_defaults().using_resolution(adapter.limits())
        } else {
            wgpu::Limits::default().using_resolution(adapter.limits())
        };
        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: Some("holo"),
                    required_features: wgpu::Features::empty(),
                    required_limits: limits,
                    memory_hints: wgpu::MemoryHints::Performance,
                },
                None,
            )
            .await
            .map_err(|e| format!("device : {e}"))?;

        let mut config = surface
            .get_default_config(&adapter, width, height)
            .ok_or_else(|| String::from("surface incompatible avec la carte graphique"))?;
        config.present_mode = wgpu::PresentMode::Fifo;
        surface.configure(&device, &config);

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("points"),
            source: wgpu::ShaderSource::Wgsl(include_str!("renderer.wgsl").into()),
        });
        let uniforms = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("uniformes"),
            size: std::mem::size_of::<Uniforms>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let instances = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("instances"),
            size: INSTANCES_MAX * std::mem::size_of::<Instance>() as u64,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("uniformes"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer { ty: wgpu::BufferBindingType::Uniform, has_dynamic_offset: false, min_binding_size: None },
                count: None,
            }],
        });
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("uniformes"),
            layout: &bind_group_layout,
            entries: &[wgpu::BindGroupEntry { binding: 0, resource: uniforms.as_entire_binding() }],
        });
        let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("points"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });
        const ATTRIBUTES: [wgpu::VertexAttribute; 3] = wgpu::vertex_attr_array![0 => Float32x2, 1 => Float32, 2 => Float32x4];
        let additive = wgpu::BlendComponent { src_factor: wgpu::BlendFactor::One, dst_factor: wgpu::BlendFactor::One, operation: wgpu::BlendOperation::Add };
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("points"),
            layout: Some(&layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs"),
                compilation_options: Default::default(),
                buffers: &[wgpu::VertexBufferLayout {
                    array_stride: std::mem::size_of::<Instance>() as u64,
                    step_mode: wgpu::VertexStepMode::Instance,
                    attributes: &ATTRIBUTES,
                }],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs"),
                compilation_options: Default::default(),
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState { color: additive, alpha: additive }),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState { topology: wgpu::PrimitiveTopology::TriangleList, ..Default::default() },
            depth_stencil: None,
            multisample: Default::default(),
            multiview: None,
            cache: None,
        });

        Ok(Renderer { surface, device, queue, config, pipeline, instances, uniforms, bind_group, backend })
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        if width > 0 && height > 0 && (width != self.config.width || height != self.config.height) {
            self.config.width = width;
            self.config.height = height;
            self.surface.configure(&self.device, &self.config);
        }
    }

    pub fn aspect(&self) -> f32 {
        self.config.width as f32 / self.config.height as f32
    }

    /// Dessine une image. Retourne le nombre de points dessinés.
    pub fn draw(&mut self, sprites: &[Sprite], time: f32) -> Result<usize, JsValue> {
        let instances: Vec<Instance> = sprites
            .iter()
            .take(INSTANCES_MAX as usize)
            .map(|s| Instance { position: [s.x, s.y], radius: s.radius, color: s.color })
            .collect();
        self.queue.write_buffer(&self.uniforms, 0, bytemuck::bytes_of(&Uniforms { aspect: self.aspect(), time, reserve: [0.0; 2] }));
        if !instances.is_empty() {
            self.queue.write_buffer(&self.instances, 0, bytemuck::cast_slice(&instances));
        }

        let image = match self.surface.get_current_texture() {
            Ok(i) => i,
            Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => {
                self.surface.configure(&self.device, &self.config);
                return Ok(0);
            }
            Err(e) => return Err(JsValue::from_str(&format!("image : {e}"))),
        };
        let view = image.texture.create_view(&Default::default());
        let mut encoder = self.device.create_command_encoder(&Default::default());
        {
            let mut passes = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("points"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations { load: wgpu::LoadOp::Clear(wgpu::Color::BLACK), store: wgpu::StoreOp::Store },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });
            passes.set_pipeline(&self.pipeline);
            passes.set_bind_group(0, &self.bind_group, &[]);
            passes.set_vertex_buffer(0, self.instances.slice(..));
            passes.draw(0..6, 0..instances.len() as u32);
        }
        self.queue.submit(Some(encoder.finish()));
        image.present();
        Ok(instances.len())
    }
}
