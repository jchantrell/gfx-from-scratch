use std::cell::RefCell;
use std::rc::Rc;

use wasm_bindgen::JsCast;
use wasm_bindgen::prelude::*;
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement, console};
mod input;

const REFLECTION_LIMIT: i32 = 4;
const FAR_CLIPPING_PLANE: f64 = f64::INFINITY;
const NEAR_CLIPPING_PLANE: f64 = 1.0;
const BACKGROUND_COLOR: Color = Color {
    r: 50,
    g: 50,
    b: 50,
};
const VIEWPORT_HEIGHT: i32 = 1;
const VIEWPORT_WIDTH: i32 = 1;

struct Canvas {
    ctx: CanvasRenderingContext2d,
    width: i32,
    height: i32,
}
impl Canvas {
    fn to_viewport(&self, x: i32, y: i32) -> Vec3 {
        let vx = x as f64 * VIEWPORT_HEIGHT as f64 / self.width as f64;
        let vy = y as f64 * VIEWPORT_WIDTH as f64 / self.height as f64;
        return Vec3 {
            x: vx,
            y: vy,
            z: NEAR_CLIPPING_PLANE,
        };
    }

    fn put_pixel(&self, x: i32, y: i32, color: Color) {
        let sx = self.width / 2 + x;
        let sy = self.height / 2 - y;
        self.ctx.set_fill_style_str(&color.serialise());
        self.ctx.fill_rect(sx as f64, sy as f64, 1.0, 1.0)
    }
}

struct Mat3 {
    m: [[f64; 3]; 3],
}
impl Mat3 {
    fn rotation_y(angle: f64) -> Mat3 {
        let c = angle.cos();
        let s = angle.sin();
        Mat3 {
            m: [[c, 0.0, s], [0.0, 1.0, 0.0], [-s, 0.0, c]],
        }
    }

    fn mul_vec3(&self, v: Vec3) -> Vec3 {
        Vec3 {
            x: self.m[0][0] * v.x + self.m[0][1] * v.y + self.m[0][2] * v.z,
            y: self.m[1][0] * v.x + self.m[1][1] * v.y + self.m[1][2] * v.z,
            z: self.m[2][0] * v.x + self.m[2][1] * v.y + self.m[2][2] * v.z,
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct Color {
    r: u8,
    g: u8,
    b: u8,
}
impl Color {
    fn serialise(&self) -> String {
        format!("rgb({},{},{})", self.r, self.g, self.b)
    }

    fn add(&self, color: Color) -> Color {
        return Color {
            r: self.r + color.r,
            g: self.g + color.g,
            b: self.b + color.b,
        };
    }

    fn mul(&self, intensity: f64) -> Color {
        return Color {
            r: (f64::from(self.r) * intensity) as u8,
            g: (f64::from(self.g) * intensity) as u8,
            b: (f64::from(self.b) * intensity) as u8,
        };
    }
}

#[derive(Debug, Clone, Copy)]
struct Vec3 {
    x: f64,
    y: f64,
    z: f64,
}
impl Vec3 {
    fn add(&self, other: Vec3) -> Vec3 {
        Vec3 {
            x: self.x + other.x,
            y: self.y + other.y,
            z: self.z + other.z,
        }
    }

    fn sub(&self, other: Vec3) -> Vec3 {
        Vec3 {
            x: self.x - other.x,
            y: self.y - other.y,
            z: self.z - other.z,
        }
    }

    fn mul(&self, scalar: f64) -> Vec3 {
        Vec3 {
            x: self.x * scalar,
            y: self.y * scalar,
            z: self.z * scalar,
        }
    }

    fn dot(&self, other: Vec3) -> f64 {
        self.x * other.x + self.y * other.y + self.z * other.z
    }

    fn len(&self) -> f64 {
        (self.x * self.x + self.y * self.y + self.z * self.z).sqrt()
    }

    fn dist(&self, other: Vec3) -> f64 {
        ((self.x - other.x).powi(2) + (self.y - other.y).powi(2) + (self.z - other.z).powi(2))
            .sqrt()
    }

    fn reverse(&self) -> Vec3 {
        Vec3 {
            x: -self.x,
            y: -self.y,
            z: -self.z,
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct Sphere {
    position: Vec3,
    radius: f64,
    radius_squared: f64,
    color: Color,
    specular: f64,
    reflective: f64,
}
impl Sphere {
    fn new(reflective: f64, specular: f64, radius: f64, position: Vec3, color: Color) -> Sphere {
        Sphere {
            position,
            radius,
            radius_squared: radius * radius,
            color,
            specular,
            reflective,
        }
    }

    fn intersect(&self, o: Vec3, d: Vec3) -> (f64, f64) {
        let co = o.sub(self.position);

        let a = d.dot(d);
        let b = 2.0 * d.dot(co);
        let c = co.dot(co) - self.radius_squared;

        let discriminant = b * b - 4.0 * a * c;
        if discriminant < 0.0 {
            return (FAR_CLIPPING_PLANE, FAR_CLIPPING_PLANE);
        }

        let t1 = (-b + discriminant.sqrt()) / (2.0 * a);
        let t2 = (-b - discriminant.sqrt()) / (2.0 * a);

        return (t1, t2);
    }
}

struct LightAmbient {
    intensity: f64,
}
struct LightPoint {
    intensity: f64,
    position: Vec3,
}
struct LightDirectional {
    intensity: f64,
    direction: Vec3,
}
struct Camera {
    position: Vec3,
    orientation: Mat3,
    rotation_y: f64,
}
impl Camera {
    fn rotate(&mut self, delta: f64) {
        self.rotation_y += delta;
        self.orientation = Mat3::rotation_y(self.rotation_y);
    }
}
struct Scene {
    camera: Camera,
    input: input::Input,
    spheres: Vec<Sphere>,
    light_ambient: LightAmbient,
    light_directional: Vec<LightDirectional>,
    light_point: Vec<LightPoint>,
}
impl Scene {
    fn update(&mut self) {
        if self.input.is_key_down("ArrowLeft") {
            self.camera.rotate(-ROTATION_SPEED);
        }
        if self.input.is_key_down("ArrowRight") {
            self.camera.rotate(ROTATION_SPEED);
        }
    }

    fn trace_ray(
        &self,
        origin: Vec3,
        destination: Vec3,
        t_min: f64,
        t_max: f64,
        depth: i32,
    ) -> Color {
        let (closest_sphere, closest_t) =
            self.closest_intersection(origin, destination, t_min, t_max);

        if closest_sphere.is_none() {
            return BACKGROUND_COLOR;
        }

        let p = origin.add(destination.mul(closest_t));
        let n = p.sub(closest_sphere.unwrap().position);
        let nn = n.mul(1.0 / n.len());

        let sphere = closest_sphere.unwrap();

        let mut light_intensity: f64 = 0.0;

        light_intensity += self.light_ambient.intensity;

        for light in &self.light_point {
            let l = light.position.sub(p);
            self.compute_lighting(
                p,
                l,
                nn,
                destination.reverse(),
                light.intensity,
                sphere.specular,
                1.0,
                &mut light_intensity,
            );
        }

        for light in &self.light_directional {
            let l = light.direction;
            self.compute_lighting(
                p,
                l,
                nn,
                destination.reverse(),
                light.intensity,
                sphere.specular,
                1.0,
                &mut light_intensity,
            );
        }

        let color = sphere.color.mul(light_intensity);

        if depth <= 0 || sphere.reflective <= 0.0 {
            return color;
        }

        let rr = self.reflect_ray(destination.reverse(), nn);
        let rc = self.trace_ray(p, rr, 0.001, FAR_CLIPPING_PLANE, depth - 1);

        return color
            .mul(1.0 - sphere.reflective)
            .add(rc.mul(sphere.reflective));
    }

    fn closest_intersection(
        &self,
        origin: Vec3,
        destination: Vec3,
        t_min: f64,
        t_max: f64,
    ) -> (Option<&Sphere>, f64) {
        let mut closest_t = t_max;
        let mut closest_sphere: Option<&Sphere> = None;

        for sphere in &self.spheres {
            let (t1, t2) = sphere.intersect(origin, destination);

            if t1 >= t_min && t1 <= t_max && t1 < closest_t {
                closest_t = t1;
                closest_sphere = Some(sphere);
            }
            if t2 >= t_min && t2 <= t_max && t2 < closest_t {
                closest_t = t2;
                closest_sphere = Some(sphere);
            }
        }

        return (closest_sphere, closest_t);
    }

    fn reflect_ray(&self, r: Vec3, n: Vec3) -> Vec3 {
        return n.mul(2.0).mul(n.dot(r)).sub(r);
    }

    fn compute_lighting(
        &self,
        p: Vec3,
        l: Vec3,
        n: Vec3,
        v: Vec3,
        intensity: f64,
        s: f64,
        t_max: f64,
        i: &mut f64,
    ) {
        // shadows
        let (shadow_sphere, _shadow_t) = self.closest_intersection(p, l, 0.001, t_max);
        if shadow_sphere.is_some() {
            return;
        }

        // diffuse
        let n_dot_l = n.dot(l);
        if n_dot_l > 0.0 {
            *i += intensity * n_dot_l / (n.len() * l.len());
        }

        // specular
        if s != -1.0 {
            let r = self.reflect_ray(l, n);
            let r_dot_v = r.dot(v);
            if r_dot_v > 0.0 {
                *i += intensity * (r_dot_v / (r.len() * v.len())).powf(s);
            }
        }
    }
}

fn create_canvas() -> Canvas {
    let window = web_sys::window().expect("no global window");
    let document = window.document().expect("no document on window");
    let dpr = window.device_pixel_ratio();

    let canvas = document
        .create_element("canvas")
        .unwrap()
        .dyn_into::<HtmlCanvasElement>()
        .unwrap();

    document.body().unwrap().append_child(&canvas).unwrap();

    let css_w = canvas.client_width();
    let css_h = canvas.client_height();

    let width = css_w * dpr as i32;
    let height = css_h * dpr as i32;

    canvas.set_width(width as u32);
    canvas.set_height(height as u32);

    let ctx = canvas
        .get_context("2d")
        .unwrap()
        .unwrap()
        .dyn_into::<CanvasRenderingContext2d>()
        .unwrap();

    ctx.scale(dpr, dpr).unwrap();

    return Canvas { ctx, width, height };
}

fn render(canvas: &Canvas, scene: &Scene) {
    for x in -canvas.width / 2..canvas.width / 2 {
        for y in -canvas.height / 2..canvas.height / 2 {
            let point = scene.camera.orientation.mul_vec3(canvas.to_viewport(x, y));
            let color = scene.trace_ray(
                scene.camera.position,
                point,
                NEAR_CLIPPING_PLANE,
                FAR_CLIPPING_PLANE,
                REFLECTION_LIMIT,
            );
            canvas.put_pixel(x, y, color);
        }
    }
}

const ROTATION_SPEED: f64 = 0.05;

#[wasm_bindgen(start)]
pub fn main() {
    let window = web_sys::window().unwrap();
    let perf = window.performance().expect("performance API unavailable");
    let canvas = Rc::new(create_canvas());

    let scene = Rc::new(RefCell::new(Scene {
        camera: Camera {
            position: Vec3 {
                x: 0.0,
                y: 0.0,
                z: -3.0,
            },
            rotation_y: 0.0,
            orientation: Mat3::rotation_y(0.0),
        },
        input: input::Input::new(),
        light_ambient: LightAmbient { intensity: 0.2 },
        light_point: vec![LightPoint {
            intensity: 0.6,
            position: Vec3 {
                x: 2.0,
                y: 1.0,
                z: 0.0,
            },
        }],
        light_directional: vec![LightDirectional {
            intensity: 0.2,
            direction: Vec3 {
                x: 1.0,
                y: 4.0,
                z: 4.0,
            },
        }],
        spheres: vec![
            Sphere::new(
                0.2,
                500.0,
                1.0,
                Vec3 {
                    x: 2.0,
                    y: 0.0,
                    z: 4.0,
                },
                Color { r: 255, g: 0, b: 0 },
            ),
            Sphere::new(
                0.3,
                500.0,
                1.0,
                Vec3 {
                    x: 0.0,
                    y: -1.0,
                    z: 3.0,
                },
                Color { r: 0, g: 0, b: 255 },
            ),
            Sphere::new(
                0.4,
                10.0,
                1.0,
                Vec3 {
                    x: -2.0,
                    y: 0.0,
                    z: 4.0,
                },
                Color { r: 0, g: 255, b: 0 },
            ),
            Sphere::new(
                0.5,
                -1.0,
                5000.0,
                Vec3 {
                    x: 0.0,
                    y: -5001.0,
                    z: 0.0,
                },
                Color {
                    r: 255,
                    g: 255,
                    b: 0,
                },
            ),
        ],
    }));

    let last_time = Rc::new(RefCell::new(perf.now()));
    let cb: Rc<RefCell<Option<Closure<dyn FnMut()>>>> = Rc::new(RefCell::new(None));
    let cb_clone = Rc::clone(&cb);

    *cb.borrow_mut() = Some(Closure::new({
        let scene = Rc::clone(&scene);
        let canvas = Rc::clone(&canvas);
        let last_time = Rc::clone(&last_time);

        move || {
            let now = perf.now();
            let dt = now - *last_time.borrow();
            *last_time.borrow_mut() = now;

            let fps = if dt > 0.0 { 1000.0 / dt } else { 0.0 };
            console::log_1(&format!("frame {:.2} ms | {:.1} fps", dt, fps).into());

            scene.borrow_mut().update();
            render(&canvas, &scene.borrow());

            window
                .request_animation_frame(
                    cb_clone.borrow().as_ref().unwrap().as_ref().unchecked_ref(),
                )
                .unwrap();
        }
    }));

    web_sys::window()
        .unwrap()
        .request_animation_frame(cb.borrow().as_ref().unwrap().as_ref().unchecked_ref())
        .unwrap();
}
